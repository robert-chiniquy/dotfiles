//! sleep-report — one line if the host slept since last look.

use std::io::{self, Read, Write};
use std::process::ExitCode;

use pr_watch::sleep::{
    catch_up_sleep, hook_additional_context, sleep_event_line, state_dir, SleepCursor,
};
use pr_watch::store::now_unix;

const USAGE: &str = "\
usage: sleep-report prime
       sleep-report --since
       sleep-report --hook

--since   print SLEEP <dur> if the laptop slept since the last look,
          then update ~/.config/sleep-report/cursor.json. Silent if none.
          First look is silent (records the cursor only).
--hook    same check; if SLEEP, print UserPromptSubmit additionalContext JSON
          (for a Grok/Claude hook). Silent if none.

State: ~/.config/sleep-report/ (override SLEEP_REPORT_STATE_DIR).
";

const PRIME: &str = "\
# sleep-report — agent primer

At the start of every turn:

    sleep-report --since

If it prints SLEEP <dur>, the laptop slept for that long since the last
look. Do not treat that wall-clock gap as a hang, a stuck poll, or a
missed GitHub event.

A UserPromptSubmit hook (`sleep-report --hook`) injects the same line
into the turn when one occurred. Still run --since if the hook is absent.

pr-watch also emits SLEEP <dur> when a blocking poll overran because the
host slept.
";

fn main() -> ExitCode {
    let argv: Vec<String> = std::env::args().collect();
    if argv.iter().any(|a| a == "-h" || a == "--help") {
        let _ = writeln!(io::stdout(), "{}", USAGE.trim_end());
        return ExitCode::SUCCESS;
    }
    if argv.get(1).map(String::as_str) == Some("prime") {
        let _ = write!(io::stdout(), "{PRIME}");
        return ExitCode::SUCCESS;
    }
    let hook = argv.iter().any(|a| a == "--hook");
    let since = hook || argv.iter().any(|a| a == "--since");
    if !since {
        let _ = writeln!(io::stderr(), "{}", USAGE.trim_end());
        return ExitCode::from(2);
    }
    if hook {
        let mut sink = String::new();
        let _ = io::stdin().read_to_string(&mut sink);
    }
    match catch_up_sleep(&state_dir()) {
        Ok(Some(d)) => {
            let line = sleep_event_line(d);
            if hook {
                let _ = writeln!(io::stdout(), "{}", hook_additional_context(&line));
            } else {
                let _ = writeln!(io::stdout(), "{line}");
            }
            ExitCode::SUCCESS
        }
        Ok(None) => ExitCode::SUCCESS,
        Err(e) => {
            // First-run or non-Darwin: record cursor so the next look works.
            if e.contains("sysctl") {
                let _ = SleepCursor {
                    last_seen: now_unix(),
                }
                .save(&state_dir());
            }
            if !hook {
                let _ = writeln!(io::stderr(), "{e}");
            }
            ExitCode::from(1)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prime_names_since() {
        assert!(PRIME.contains("--since"));
        assert!(PRIME.contains("SLEEP"));
    }
}
