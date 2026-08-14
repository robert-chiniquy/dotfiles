//! pr-watch — one line per PR state change, then exit on a terminal event.

use std::io::{self, Write};
use std::process::ExitCode;
use std::time::{Duration, Instant};

use pr_watch::store::{
    canonical_cwd, catch_up_lines, looks_like_stamp, now_unix, parse_since_stamp, record_event,
    state_dir, Store,
};
use pr_watch::{fetch_snapshot, parse_target, sleep_interval, Until};

const USAGE: &str = "\
usage: pr-watch prime
       pr-watch [--repo owner/repo] [--cwd DIR] [--interval SECS]
                [--until action|merged|never] [--once] [--since [TIME]]
                [--emit-snapshot] [--max-wait SECS]
                owner/repo#N | https://github.com/owner/repo/pull/N | N

Prints one line per change (CI GREEN, CI RED <check>, REVIEW CHANGES_REQUESTED,
REVIEW APPROVED, THREADS N, MERGED, CLOSED).

--since [TIME]   catch-up vs last read for this cwd+repo+PR (default TIME is
                 that last-read). First look prints the current snapshot.
                 One-shot unless --until is also passed.
--once           print the current snapshot (also records the cursor)
--until action   (default when watching) exit on CI green/red, CHANGES_REQUESTED,
                 merged, closed
--until merged   exit only on merged (0) or closed (1)
--until never    run until --max-wait or signal

State: ~/.config/pr-watch/cursors.json (override PR_WATCH_STATE_DIR).
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
        | REVIEW REQUIRED | THREADS N | MERGED | CLOSED
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

enum ParseOutcome {
    Args(Args),
    Help,
    Prime,
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
    let mut i = 1;
    while i < argv.len() {
        match argv[i].as_str() {
            "-h" | "--help" => return Ok(ParseOutcome::Help),
            "prime" if positional.is_empty() => return Ok(ParseOutcome::Prime),
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
    let spec = match positional.len() {
        1 => positional[0].clone(),
        2 => format!("{}#{}", positional[0], positional[1]),
        _ => return Err(USAGE.trim_end().to_string()),
    };
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

fn persist(store: &Store) -> Result<(), String> {
    store.save(&state_dir())
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
    let first_lines = if args.since {
        catch_up_lines(&store, &cwd, &target, &first, now_unix(), args.since_time)
    } else if args.once || args.emit_snapshot {
        first.snapshot_lines()
    } else {
        Vec::new()
    };
    if !first_lines.is_empty() && emit(&first_lines).is_err() {
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
                let _ = writeln!(io::stdout(), "FAILED timeout");
                let _ = io::stdout().flush();
                return ExitCode::from(1);
            }
        }
        sleep_interval(args.interval);
        let next = match fetch_snapshot(&target) {
            Ok(s) => s,
            Err(e) => {
                let _ = writeln!(io::stderr(), "{e}");
                continue;
            }
        };
        let events = next.events_since(&prev);
        if !events.is_empty() && emit(&events).is_err() {
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
        assert!(PRIME.contains("~/.config/pr-watch/"));
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
