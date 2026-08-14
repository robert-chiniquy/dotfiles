//! pr-watch — one line per PR state change, then exit on a terminal event.

use std::io::{self, Write};
use std::process::ExitCode;
use std::time::{Duration, Instant, SystemTime};

use pr_watch::config::{Config, CopilotMode};
use pr_watch::sleep::{sleep_event_line, sleep_overrun};
use pr_watch::store::{
    canonical_cwd, catch_up_lines, looks_like_stamp, now_unix, parse_since_stamp, record_event,
    state_dir, Store,
};
use pr_watch::{
    ensure_copilot_gate, fetch_snapshot, next_actions, parse_target, request_copilot,
    sleep_interval, Snapshot, Until,
};

const USAGE: &str = "\
usage: pr-watch prime
       pr-watch request-copilot [--repo owner/repo] owner/repo#N | N
       pr-watch set copilot request|skip [--cwd DIR]
       pr-watch skip-copilot [--cwd DIR]
       pr-watch unskip-copilot [--cwd DIR]
       pr-watch [--repo owner/repo] [--cwd DIR] [--interval SECS]
                [--until action|merged|never] [--once] [--since [TIME]]
                [--emit-snapshot] [--max-wait SECS]
                owner/repo#N | https://github.com/owner/repo/pull/N | N

Prints one line per change (CI GREEN, CI RED <check>, REVIEW CHANGES_REQUESTED,
REVIEW APPROVED, THREADS N, COPILOT NONE|REQUESTED|REVIEWED|CHANGES_REQUESTED,
SLEEP <dur>, MERGED, CLOSED), then NEXT lines for each kind.

--since [TIME]   catch-up vs last read for this cwd+repo+PR (default TIME is
                 that last-read). First look prints the current snapshot.
                 One-shot unless --until is also passed.
--once           print the current snapshot (also records the cursor)
--until action   (default when watching) exit on CI green/red, CHANGES_REQUESTED,
                 COPILOT NONE/REVIEWED/CHANGES_REQUESTED, merged, closed
--until merged   exit only on merged (0) or closed (1)
--until never    run until --max-wait or signal

request-copilot  POST github-copilot as a reviewer. Do this before asking a
                 human. Do not invent a different reviewer login.
set copilot      write ~/.config/pr-watch/config.yaml. Without --cwd, sets the
                 global default (request|skip). With --cwd, overrides that dir.
skip-copilot     cwd override to skip (default cwd: pwd).
unskip-copilot   drop the cwd override (inherit the global default).

State: ~/.config/pr-watch/ (override PR_WATCH_STATE_DIR).
";

const PRIME: &str = "\
# pr-watch — agent primer

Catch-up (second and later looks; does not miss transitions since last read):

    pr-watch --since owner/repo#N

First use for that cwd+repo+PR prints the current snapshot and records it
under ~/.config/pr-watch/. Later --since prints only diffs.

Do not use --once or gh pr view as the second look. --once is a full
snapshot and hides what changed. --since is the catch-up.

Live wait (blocking; first poll silent unless --since):

    pr-watch --until action owner/repo#N
    pr-watch --since --until action owner/repo#N

Harness monitor: wrap the same command. Each stdout line is an event.
MUST NOT start a shell background job (cmd &, nohup, $!).

Events: CI GREEN | CI RED <check> | REVIEW CHANGES_REQUESTED | REVIEW APPROVED
        | REVIEW REQUIRED | THREADS N | COPILOT NONE | COPILOT REQUESTED
        | COPILOT REVIEWED | COPILOT CHANGES_REQUESTED | MERGED | CLOSED

Each event is followed by NEXT <KIND> <steps>. Follow those steps. Example:
REVIEW CHANGES_REQUESTED -> read threads; fix if reasonable else ask; after
each pushed fix reply Addressed in <sha> and resolve; then pr-watch --until action.

Copilot gate: MUST request Copilot and wait for COPILOT REVIEWED before
asking a human for a review. If COPILOT NONE:

    pr-watch request-copilot owner/repo#N
    pr-watch --until action owner/repo#N

Do not request a human while COPILOT is NONE or REQUESTED. Address
COPILOT CHANGES_REQUESTED like any other changes-requested. Login is
github-copilot (not copilot-pull-request-reviewer).

Copilot default lives in ~/.config/pr-watch/config.yaml (`copilot:
request` or `skip`). A `cwd` map overrides that per canonical working
directory. COPILOT SKIP replaces the gate when the resolved mode is skip.

    pr-watch set copilot request
    pr-watch set copilot skip --cwd /path/to/checkout
    pr-watch skip-copilot
    pr-watch unskip-copilot

Do not skip unless that conclusion is recorded. Host sleep: SLEEP <dur>
from a blocking poll, or `sleep-report --since` at the start of each turn.
";

struct Args {
    spec: String,
    repo: Option<String>,
    cwd: Option<String>,
    interval: Duration,
    until: Until,
    until_set: bool,
    once: bool,
    since: bool,
    since_time: Option<i64>,
    emit_snapshot: bool,
    max_wait: Option<Duration>,
}

enum CopilotScope {
    Global,
    Cwd(Option<String>),
}

enum ParseOutcome {
    Args(Args),
    Help,
    Prime,
    RequestCopilot {
        spec: String,
        repo: Option<String>,
    },
    SetCopilot {
        mode: CopilotMode,
        scope: CopilotScope,
    },
    ClearCwd {
        cwd: Option<String>,
    },
}

fn parse_args(argv: &[String]) -> Result<ParseOutcome, String> {
    let mut repo = None;
    let mut cwd = None;
    let mut interval = Duration::from_secs(20);
    let mut until = Until::Action;
    let mut until_set = false;
    let mut once = false;
    let mut since = false;
    let mut since_time = None;
    let mut emit_snapshot = false;
    let mut max_wait = None;
    let mut positional = Vec::new();
    let mut request_copilot_cmd = false;
    let mut skip_copilot_cmd = false;
    let mut unskip_copilot_cmd = false;
    let mut set_copilot = None;
    let mut i = 1;
    while i < argv.len() {
        match argv[i].as_str() {
            "-h" | "--help" => return Ok(ParseOutcome::Help),
            "prime" if positional.is_empty() && !request_copilot_cmd => {
                return Ok(ParseOutcome::Prime)
            }
            "request-copilot" if positional.is_empty() && !request_copilot_cmd => {
                request_copilot_cmd = true;
            }
            "skip-copilot" if positional.is_empty() && !request_copilot_cmd => {
                skip_copilot_cmd = true;
            }
            "unskip-copilot" if positional.is_empty() && !request_copilot_cmd => {
                unskip_copilot_cmd = true;
            }
            "set" if positional.is_empty() && set_copilot.is_none() => {
                i += 1;
                let key = argv
                    .get(i)
                    .ok_or("set needs copilot request|skip")?
                    .as_str();
                if key != "copilot" {
                    return Err(format!("set {key} is unknown (want set copilot)"));
                }
                i += 1;
                set_copilot = Some(CopilotMode::parse(
                    argv.get(i).ok_or("set copilot needs request|skip")?,
                )?);
            }
            "--once" => once = true,
            "--emit-snapshot" => emit_snapshot = true,
            "--since" => {
                since = true;
                if let Some(next) = argv.get(i + 1) {
                    if looks_like_stamp(next) {
                        i += 1;
                        since_time = Some(parse_since_stamp(next)?);
                    }
                }
            }
            flag if flag.starts_with("--since=") => {
                since = true;
                since_time = Some(parse_since_stamp(&flag[8..])?);
            }
            "--repo" => {
                i += 1;
                repo = Some(argv.get(i).ok_or("--repo needs owner/repo")?.clone());
            }
            "--cwd" => {
                i += 1;
                cwd = Some(argv.get(i).ok_or("--cwd needs a directory")?.clone());
            }
            "--interval" => {
                i += 1;
                let n: u64 = argv
                    .get(i)
                    .ok_or("--interval needs seconds")?
                    .parse()
                    .map_err(|_| "interval must be seconds")?;
                if n == 0 {
                    return Err("interval must be > 0".into());
                }
                interval = Duration::from_secs(n);
            }
            "--until" => {
                i += 1;
                until = Until::parse(argv.get(i).ok_or("--until needs action|merged|never")?)?;
                until_set = true;
            }
            "--max-wait" => {
                i += 1;
                let n: u64 = argv
                    .get(i)
                    .ok_or("--max-wait needs seconds")?
                    .parse()
                    .map_err(|_| "max-wait must be seconds")?;
                max_wait = Some(Duration::from_secs(n));
            }
            other if other.starts_with('-') => {
                return Err(format!("unknown flag {other}\n{USAGE}"));
            }
            other => positional.push(other.to_string()),
        }
        i += 1;
    }
    if skip_copilot_cmd || unskip_copilot_cmd || set_copilot.is_some() {
        if !positional.is_empty() {
            return Err(
                "copilot config commands do not take a PR spec; use --cwd DIR to override a directory"
                    .into(),
            );
        }
        if let Some(mode) = set_copilot {
            let scope = if cwd.is_some() {
                CopilotScope::Cwd(cwd)
            } else {
                CopilotScope::Global
            };
            return Ok(ParseOutcome::SetCopilot { mode, scope });
        }
        if skip_copilot_cmd {
            return Ok(ParseOutcome::SetCopilot {
                mode: CopilotMode::Skip,
                scope: CopilotScope::Cwd(cwd),
            });
        }
        return Ok(ParseOutcome::ClearCwd { cwd });
    }
    let spec = match positional.len() {
        1 => positional[0].clone(),
        2 => format!("{}#{}", positional[0], positional[1]),
        _ => return Err(USAGE.trim_end().to_string()),
    };
    if request_copilot_cmd {
        return Ok(ParseOutcome::RequestCopilot { spec, repo });
    }
    Ok(ParseOutcome::Args(Args {
        spec,
        repo,
        cwd,
        interval,
        until,
        until_set,
        once,
        since,
        since_time,
        emit_snapshot,
        max_wait,
    }))
}

fn emit(lines: &[String]) -> io::Result<()> {
    let mut out = io::stdout();
    for line in lines {
        writeln!(out, "{line}")?;
    }
    out.flush()
}

fn emit_with_next(
    lines: &[String],
    spec: &str,
    now: &Snapshot,
    copilot_required: bool,
) -> io::Result<()> {
    if lines.is_empty() {
        return Ok(());
    }
    emit(lines)?;
    emit(&next_actions(lines, spec, now, copilot_required))
}

fn run_set_copilot(mode: CopilotMode, scope: CopilotScope) -> ExitCode {
    let dir = state_dir();
    let mut cfg = match Config::load(&dir) {
        Ok(c) => c,
        Err(e) => {
            let _ = writeln!(io::stderr(), "{e}");
            return ExitCode::from(1);
        }
    };
    let line = match scope {
        CopilotScope::Global => {
            cfg.set_global(mode);
            format!("COPILOT {}", mode.as_str().to_ascii_uppercase())
        }
        CopilotScope::Cwd(explicit) => {
            let cwd = match canonical_cwd(explicit.as_deref()) {
                Ok(c) => c,
                Err(e) => {
                    let _ = writeln!(io::stderr(), "{e}");
                    return ExitCode::from(2);
                }
            };
            cfg.set_cwd(&cwd, mode);
            format!("COPILOT {} {cwd}", mode.as_str().to_ascii_uppercase())
        }
    };
    if let Err(e) = cfg.save(&dir) {
        let _ = writeln!(io::stderr(), "{e}");
        return ExitCode::from(1);
    }
    let _ = writeln!(io::stdout(), "{line}");
    ExitCode::SUCCESS
}

fn run_clear_cwd(explicit: Option<&str>) -> ExitCode {
    let cwd = match canonical_cwd(explicit) {
        Ok(c) => c,
        Err(e) => {
            let _ = writeln!(io::stderr(), "{e}");
            return ExitCode::from(2);
        }
    };
    let dir = state_dir();
    let mut cfg = match Config::load(&dir) {
        Ok(c) => c,
        Err(e) => {
            let _ = writeln!(io::stderr(), "{e}");
            return ExitCode::from(1);
        }
    };
    cfg.clear_cwd(&cwd);
    if let Err(e) = cfg.save(&dir) {
        let _ = writeln!(io::stderr(), "{e}");
        return ExitCode::from(1);
    }
    let _ = writeln!(io::stdout(), "COPILOT DEFAULT {cwd}");
    ExitCode::SUCCESS
}

fn persist(store: &Store) -> Result<(), String> {
    store.save(&state_dir())
}

fn run_request_copilot(spec: &str, repo: Option<&str>) -> ExitCode {
    let target = match parse_target(spec, repo) {
        Ok(t) => t,
        Err(e) => {
            let _ = writeln!(io::stderr(), "{e}");
            return ExitCode::from(2);
        }
    };
    if let Err(e) = request_copilot(&target) {
        let _ = writeln!(io::stderr(), "{e}");
        return ExitCode::from(1);
    }
    let spec = target.to_string();
    let lines = vec!["COPILOT REQUESTED".to_string()];
    let now = Snapshot {
        state: pr_watch::PrState::Open,
        ci: pr_watch::Ci::Pending,
        ci_failed: None,
        review: pr_watch::Review::None,
        threads: 0,
        copilot: pr_watch::Copilot::Requested,
    };
    if emit_with_next(&lines, &spec, &now, true).is_err() {
        return ExitCode::from(1);
    }
    ExitCode::SUCCESS
}

fn main() -> ExitCode {
    let argv: Vec<String> = std::env::args().collect();
    let args = match parse_args(&argv) {
        Ok(ParseOutcome::Args(a)) => a,
        Ok(ParseOutcome::Help) => {
            let _ = writeln!(io::stdout(), "{}", USAGE.trim_end());
            return ExitCode::SUCCESS;
        }
        Ok(ParseOutcome::Prime) => {
            let _ = write!(io::stdout(), "{PRIME}");
            return ExitCode::SUCCESS;
        }
        Ok(ParseOutcome::RequestCopilot { spec, repo }) => {
            return run_request_copilot(&spec, repo.as_deref());
        }
        Ok(ParseOutcome::SetCopilot { mode, scope }) => {
            return run_set_copilot(mode, scope);
        }
        Ok(ParseOutcome::ClearCwd { cwd }) => {
            return run_clear_cwd(cwd.as_deref());
        }
        Err(e) => {
            let _ = writeln!(io::stderr(), "{e}");
            return ExitCode::from(2);
        }
    };
    let target = match parse_target(&args.spec, args.repo.as_deref()) {
        Ok(t) => t,
        Err(e) => {
            let _ = writeln!(io::stderr(), "{e}");
            return ExitCode::from(2);
        }
    };
    let cwd = match canonical_cwd(args.cwd.as_deref()) {
        Ok(c) => c,
        Err(e) => {
            let _ = writeln!(io::stderr(), "{e}");
            return ExitCode::from(2);
        }
    };
    let mut store = match Store::load(&state_dir()) {
        Ok(s) => s,
        Err(e) => {
            let _ = writeln!(io::stderr(), "{e}");
            return ExitCode::from(1);
        }
    };

    let started = Instant::now();
    let first = match fetch_snapshot(&target) {
        Ok(s) => s,
        Err(e) => {
            let _ = writeln!(io::stderr(), "{e}");
            return ExitCode::from(1);
        }
    };

    let oneshot = args.once || (args.since && !args.until_set);
    let spec = target.to_string();
    let cfg = match Config::load(&state_dir()) {
        Ok(c) => c,
        Err(e) => {
            let _ = writeln!(io::stderr(), "{e}");
            return ExitCode::from(1);
        }
    };
    let copilot_required = cfg.copilot_required(&cwd);
    let mut first_lines = if args.since {
        catch_up_lines(&store, &cwd, &target, &first, now_unix(), args.since_time)
    } else if args.once || args.emit_snapshot {
        first.snapshot_lines()
    } else {
        Vec::new()
    };
    ensure_copilot_gate(&mut first_lines, &first, copilot_required);
    if emit_with_next(&first_lines, &spec, &first, copilot_required).is_err() {
        return ExitCode::from(1);
    }
    record_event(
        &mut store,
        &cwd,
        &target,
        first.clone(),
        now_unix(),
        !first_lines.is_empty(),
    );
    if let Err(e) = persist(&store) {
        let _ = writeln!(io::stderr(), "{e}");
        return ExitCode::from(1);
    }
    if oneshot {
        return ExitCode::SUCCESS;
    }
    if args.since {
        if let Some(code) = first.until_hit(args.until, &first_lines) {
            return ExitCode::from(code as u8);
        }
    }

    let mut prev = first;
    loop {
        if let Some(max) = args.max_wait {
            if started.elapsed() >= max {
                let _ = emit_with_next(&["FAILED timeout".into()], &spec, &prev, copilot_required);
                return ExitCode::from(1);
            }
        }
        let mono0 = Instant::now();
        let wall0 = SystemTime::now();
        sleep_interval(args.interval);
        let wall = SystemTime::now().duration_since(wall0).unwrap_or_default();
        let mono = mono0.elapsed();
        if let Some(d) = sleep_overrun(args.interval, wall, mono) {
            let line = sleep_event_line(d);
            if emit_with_next(&[line], &spec, &prev, copilot_required).is_err() {
                return ExitCode::from(1);
            }
        }
        let next = match fetch_snapshot(&target) {
            Ok(s) => s,
            Err(e) => {
                let _ = writeln!(io::stderr(), "{e}");
                continue;
            }
        };
        let mut events = next.events_since(&prev);
        if !copilot_required {
            events.retain(|l| !l.starts_with("COPILOT "));
        }
        if emit_with_next(&events, &spec, &next, copilot_required).is_err() {
            return ExitCode::from(1);
        }
        record_event(
            &mut store,
            &cwd,
            &target,
            next.clone(),
            now_unix(),
            !events.is_empty(),
        );
        if let Err(e) = persist(&store) {
            let _ = writeln!(io::stderr(), "{e}");
        }
        if let Some(code) = next.until_hit(args.until, &events) {
            return ExitCode::from(code as u8);
        }
        prev = next;
    }
}

#[cfg(test)]
mod cli_tests {
    use super::*;

    #[test]
    fn parse_until_and_repo() {
        let argv = vec![
            "pr-watch".into(),
            "--repo".into(),
            "ductone/multipass".into(),
            "--until".into(),
            "merged".into(),
            "627".into(),
        ];
        let ParseOutcome::Args(a) = parse_args(&argv).unwrap() else {
            panic!("expected args");
        };
        assert_eq!(a.spec, "627");
        assert_eq!(a.repo.as_deref(), Some("ductone/multipass"));
        assert_eq!(a.until, Until::Merged);
        assert!(a.until_set);
    }

    #[test]
    fn parse_since_without_time() {
        let argv = vec!["pr-watch".into(), "--since".into(), "ductone/c1#1".into()];
        let ParseOutcome::Args(a) = parse_args(&argv).unwrap() else {
            panic!("expected args");
        };
        assert!(a.since);
        assert!(a.since_time.is_none());
        assert!(!a.until_set);
    }

    #[test]
    fn parse_since_with_rfc3339() {
        let argv = vec![
            "pr-watch".into(),
            "--since".into(),
            "2026-08-14T18:00:00Z".into(),
            "ductone/c1#1".into(),
        ];
        let ParseOutcome::Args(a) = parse_args(&argv).unwrap() else {
            panic!("expected args");
        };
        assert!(a.since);
        assert!(a.since_time.is_some());
    }

    #[test]
    fn parse_prime() {
        let argv = vec!["pr-watch".into(), "prime".into()];
        assert!(matches!(parse_args(&argv), Ok(ParseOutcome::Prime)));
    }

    #[test]
    fn prime_names_since_and_changes_requested() {
        assert!(PRIME.contains("--since"));
        assert!(PRIME.contains("CHANGES_REQUESTED"));
        assert!(PRIME.contains("NEXT"));
        assert!(PRIME.contains("~/.config/pr-watch/"));
        assert!(PRIME.contains("request-copilot"));
        assert!(PRIME.contains("COPILOT REVIEWED"));
        assert!(PRIME.contains("github-copilot"));
        assert!(PRIME.contains("skip-copilot"));
        assert!(PRIME.contains("config.yaml"));
        assert!(PRIME.contains("sleep-report"));
    }

    #[test]
    fn parse_skip_copilot_is_cwd_scope() {
        let argv = vec!["pr-watch".into(), "skip-copilot".into()];
        let ParseOutcome::SetCopilot { mode, scope } = parse_args(&argv).unwrap() else {
            panic!("expected set-copilot");
        };
        assert_eq!(mode, CopilotMode::Skip);
        assert!(matches!(scope, CopilotScope::Cwd(None)));
    }

    #[test]
    fn parse_set_copilot_global() {
        let argv = vec![
            "pr-watch".into(),
            "set".into(),
            "copilot".into(),
            "skip".into(),
        ];
        let ParseOutcome::SetCopilot { mode, scope } = parse_args(&argv).unwrap() else {
            panic!("expected set-copilot");
        };
        assert_eq!(mode, CopilotMode::Skip);
        assert!(matches!(scope, CopilotScope::Global));
    }

    #[test]
    fn parse_set_copilot_cwd() {
        let argv = vec![
            "pr-watch".into(),
            "set".into(),
            "copilot".into(),
            "request".into(),
            "--cwd".into(),
            "/work/docs".into(),
        ];
        let ParseOutcome::SetCopilot { mode, scope } = parse_args(&argv).unwrap() else {
            panic!("expected set-copilot");
        };
        assert_eq!(mode, CopilotMode::Request);
        match scope {
            CopilotScope::Cwd(Some(p)) => assert_eq!(p, "/work/docs"),
            CopilotScope::Cwd(None) => panic!("expected explicit cwd"),
            CopilotScope::Global => panic!("expected cwd override"),
        }
    }

    #[test]
    fn parse_skip_rejects_pr_spec() {
        let argv = vec![
            "pr-watch".into(),
            "skip-copilot".into(),
            "ductone/docs".into(),
        ];
        assert!(parse_args(&argv).is_err());
    }

    #[test]
    fn parse_request_copilot() {
        let argv = vec![
            "pr-watch".into(),
            "request-copilot".into(),
            "ductone/multipass#627".into(),
        ];
        let ParseOutcome::RequestCopilot { spec, repo } = parse_args(&argv).unwrap() else {
            panic!("expected request-copilot");
        };
        assert_eq!(spec, "ductone/multipass#627");
        assert!(repo.is_none());
    }

    #[test]
    fn parse_request_copilot_with_repo_flag() {
        let argv = vec![
            "pr-watch".into(),
            "request-copilot".into(),
            "--repo".into(),
            "ductone/c1".into(),
            "23321".into(),
        ];
        let ParseOutcome::RequestCopilot { spec, repo } = parse_args(&argv).unwrap() else {
            panic!("expected request-copilot");
        };
        assert_eq!(spec, "23321");
        assert_eq!(repo.as_deref(), Some("ductone/c1"));
    }

    #[test]
    fn parse_rejects_zero_interval() {
        let argv = vec![
            "pr-watch".into(),
            "--interval".into(),
            "0".into(),
            "ductone/c1#1".into(),
        ];
        assert!(parse_args(&argv).is_err());
    }
}
