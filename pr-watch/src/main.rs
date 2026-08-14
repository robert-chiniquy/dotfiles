//! pr-watch — one line per PR state change, then exit on a terminal event.

use std::io::{self, Write};
use std::process::ExitCode;
use std::time::{Duration, Instant};

use pr_watch::{fetch_snapshot, parse_target, sleep_interval, Until};

const USAGE: &str = "\
usage: pr-watch [--repo owner/repo] [--interval SECS] [--until action|merged|never]
                [--once] [--emit-snapshot] [--max-wait SECS]
                owner/repo#N | https://github.com/owner/repo/pull/N | N

Prints one line per change (CI GREEN, CI RED <check>, REVIEW CHANGES_REQUESTED,
REVIEW APPROVED, THREADS N, MERGED, CLOSED). First poll is the baseline and is
silent unless --once or --emit-snapshot.

--until action   (default) exit on CI green/red, CHANGES_REQUESTED, merged, closed
--until merged   exit only on merged (0) or closed (1)
--until never    run until --max-wait or signal
";

struct Args {
    spec: String,
    repo: Option<String>,
    interval: Duration,
    until: Until,
    once: bool,
    emit_snapshot: bool,
    max_wait: Option<Duration>,
}

enum ParseOutcome {
    Args(Args),
    Help,
}

fn parse_args(argv: &[String]) -> Result<ParseOutcome, String> {
    let mut repo = None;
    let mut interval = Duration::from_secs(20);
    let mut until = Until::Action;
    let mut once = false;
    let mut emit_snapshot = false;
    let mut max_wait = None;
    let mut positional = Vec::new();
    let mut i = 1;
    while i < argv.len() {
        match argv[i].as_str() {
            "-h" | "--help" => return Ok(ParseOutcome::Help),
            "--once" => once = true,
            "--emit-snapshot" => emit_snapshot = true,
            "--repo" => {
                i += 1;
                repo = Some(argv.get(i).ok_or("--repo needs owner/repo")?.clone());
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
        interval,
        until,
        once,
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

fn main() -> ExitCode {
    let argv: Vec<String> = std::env::args().collect();
    let args = match parse_args(&argv) {
        Ok(ParseOutcome::Args(a)) => a,
        Ok(ParseOutcome::Help) => {
            let _ = writeln!(io::stdout(), "{}", USAGE.trim_end());
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

    let started = Instant::now();
    let first = match fetch_snapshot(&target) {
        Ok(s) => s,
        Err(e) => {
            let _ = writeln!(io::stderr(), "{e}");
            return ExitCode::from(1);
        }
    };

    if args.once || args.emit_snapshot {
        if emit(&first.snapshot_lines()).is_err() {
            return ExitCode::from(1);
        }
        if args.once {
            return ExitCode::SUCCESS;
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
