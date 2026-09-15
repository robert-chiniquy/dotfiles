//! Host sleep detection for agent turns and blocking watches.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::Duration;

use crate::store::now_unix;

/// Minimum gap between `--since` / `--hook` looks. A second invocation
/// inside this window exits without sysctl or cursor writes.
pub const LOOK_MIN_INTERVAL_SECS: i64 = 10;

#[derive(Debug, Default, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct SleepCursor {
    pub last_seen: i64,
    /// Unix seconds of the last `--since` / `--hook` look. Distinct from
    /// `last_seen` so a skipped rerun does not advance the sleep cursor.
    #[serde(default)]
    pub last_run: i64,
}

pub fn state_dir() -> PathBuf {
    if let Ok(p) = std::env::var("SLEEP_REPORT_STATE_DIR") {
        return PathBuf::from(p);
    }
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
    PathBuf::from(home).join(".config/sleep-report")
}

impl SleepCursor {
    pub fn path_in(dir: &Path) -> PathBuf {
        dir.join("cursor.json")
    }

    pub fn load(dir: &Path) -> Result<Self, String> {
        let path = Self::path_in(dir);
        if !path.exists() {
            return Ok(Self::default());
        }
        let raw = fs::read_to_string(&path).map_err(|e| format!("read {}: {e}", path.display()))?;
        if raw.trim().is_empty() {
            return Ok(Self::default());
        }
        serde_json::from_str(&raw).map_err(|e| format!("parse {}: {e}", path.display()))
    }

    pub fn save(&self, dir: &Path) -> Result<(), String> {
        fs::create_dir_all(dir).map_err(|e| format!("mkdir {}: {e}", dir.display()))?;
        let path = Self::path_in(dir);
        let tmp = dir.join("cursor.json.tmp");
        let body = serde_json::to_string_pretty(self).map_err(|e| format!("encode cursor: {e}"))?;
        fs::write(&tmp, &body).map_err(|e| format!("write {}: {e}", tmp.display()))?;
        let parsed: SleepCursor =
            serde_json::from_str(&body).map_err(|e| format!("reparse cursor: {e}"))?;
        if parsed.last_seen != self.last_seen || parsed.last_run != self.last_run {
            let _ = fs::remove_file(&tmp);
            return Err("sleep cursor write lost last_seen or last_run".into());
        }
        fs::rename(&tmp, &path).map_err(|e| format!("rename {}: {e}", path.display()))
    }
}

/// Compact duration for event lines: `12m 4s`, `1h 2m`.
pub fn fmt_dur(d: Duration) -> String {
    let s = d.as_secs();
    let (h, s) = (s / 3600, s % 3600);
    let (m, s) = (s / 60, s % 60);
    match (h, m, s) {
        (0, 0, s) => format!("{s}s"),
        (0, m, 0) => format!("{m}m"),
        (0, m, s) => format!("{m}m {s}s"),
        (h, 0, 0) => format!("{h}h"),
        (h, m, 0) => format!("{h}h {m}m"),
        (h, m, s) => format!("{h}h {m}m {s}s"),
    }
}

/// Instant pauses across macOS sleep; SystemTime does not. `wall - mono`
/// is the sleep gap. Ignore jitter under 5s.
pub fn sleep_overrun(planned: Duration, wall: Duration, mono: Duration) -> Option<Duration> {
    if wall <= planned.saturating_add(Duration::from_secs(2)) {
        return None;
    }
    let slept = wall.saturating_sub(mono);
    if slept < Duration::from_secs(5) {
        None
    } else {
        Some(slept)
    }
}

/// Last completed Sleep→Wake from `kern.sleeptime` / `kern.waketime`, if
/// the wake is after `last_seen`.
pub fn last_sleep_after(last_seen: i64, sleep_sec: i64, wake_sec: i64) -> Option<Duration> {
    if sleep_sec <= 0 || wake_sec <= 0 || wake_sec <= sleep_sec {
        return None;
    }
    if wake_sec <= last_seen {
        return None;
    }
    Some(Duration::from_secs((wake_sec - sleep_sec) as u64))
}

pub fn parse_sysctl_sec(line: &str) -> Option<i64> {
    let rest = line.split("sec = ").nth(1)?;
    let num = rest
        .split(|c: char| !c.is_ascii_digit() && c != '-')
        .find(|s| !s.is_empty())?;
    num.parse().ok()
}

pub fn read_kern_sleep_wake() -> Result<(i64, i64), String> {
    let bin = std::env::var("SLEEP_REPORT_SYSCTL").unwrap_or_else(|_| "sysctl".into());
    let out = Command::new(&bin)
        .args(["kern.sleeptime", "kern.waketime"])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .map_err(|e| format!("exec {bin}: {e}"))?;
    if !out.status.success() {
        let err = String::from_utf8_lossy(&out.stderr);
        return Err(format!("{bin} failed: {}", err.trim()));
    }
    let text = String::from_utf8(out.stdout).map_err(|e| format!("sysctl stdout: {e}"))?;
    let mut sleep_sec = None;
    let mut wake_sec = None;
    for line in text.lines() {
        if line.contains("sleeptime") {
            sleep_sec = parse_sysctl_sec(line);
        } else if line.contains("waketime") {
            wake_sec = parse_sysctl_sec(line);
        }
    }
    match (sleep_sec, wake_sec) {
        (Some(s), Some(w)) => Ok((s, w)),
        _ => Err("sysctl missing kern.sleeptime or kern.waketime".into()),
    }
}

/// True when `now` is strictly less than 10s after `last_run`.
/// `last_run == 0` means never looked; that is never too soon.
pub fn look_is_too_soon(last_run: i64, now: i64) -> bool {
    last_run > 0 && now >= last_run && now - last_run < LOOK_MIN_INTERVAL_SECS
}

/// Catch-up vs stored last_seen. First look records the cursor and is silent
/// (does not replay the last sleep from before we started tracking).
/// A look within 10s of the previous `--since`/`--hook` returns `Ok(None)`
/// without sysctl and without moving `last_seen`.
pub fn catch_up_sleep(dir: &Path) -> Result<Option<Duration>, String> {
    let mut cur = SleepCursor::load(dir)?;
    let now = now_unix();
    if look_is_too_soon(cur.last_run, now) {
        return Ok(None);
    }
    cur.last_run = now;
    cur.save(dir)?;
    let first = cur.last_seen == 0;
    let (sleep_sec, wake_sec) = read_kern_sleep_wake()?;
    let slept = if first {
        None
    } else {
        last_sleep_after(cur.last_seen, sleep_sec, wake_sec)
    };
    cur.last_seen = now;
    cur.save(dir)?;
    Ok(slept)
}

pub fn sleep_event_line(d: Duration) -> String {
    format!("SLEEP {}", fmt_dur(d))
}

pub fn hook_additional_context(line: &str) -> String {
    format!(
        "{{\"hookSpecificOutput\":{{\"hookEventName\":\"UserPromptSubmit\",\"additionalContext\":{}}}}}",
        serde_json::to_string(&format!(
            "{line}. Host slept since last turn. Do not treat that wall-clock gap as a hang or a missed poll."
        ))
        .unwrap_or_else(|_| "\"SLEEP\"".into())
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fmt_dur_compacts() {
        assert_eq!(fmt_dur(Duration::from_secs(4)), "4s");
        assert_eq!(fmt_dur(Duration::from_secs(60)), "1m");
        assert_eq!(fmt_dur(Duration::from_secs(65)), "1m 5s");
        assert_eq!(fmt_dur(Duration::from_secs(3661)), "1h 1m 1s");
    }

    #[test]
    fn overrun_ignores_jitter() {
        assert!(sleep_overrun(
            Duration::from_secs(20),
            Duration::from_secs(21),
            Duration::from_secs(20)
        )
        .is_none());
        let d = sleep_overrun(
            Duration::from_secs(20),
            Duration::from_secs(620),
            Duration::from_secs(20),
        )
        .unwrap();
        assert_eq!(d, Duration::from_secs(600));
    }

    #[test]
    fn last_sleep_only_after_cursor() {
        assert!(last_sleep_after(100, 50, 80).is_none());
        assert_eq!(
            last_sleep_after(100, 90, 160),
            Some(Duration::from_secs(70))
        );
        assert!(last_sleep_after(100, 0, 0).is_none());
        assert!(last_sleep_after(100, 180, 160).is_none());
    }

    #[test]
    fn parse_darwin_sysctl_line() {
        let line = "kern.sleeptime: { sec = 1786742817, usec = 700395 } Fri Aug 14 14:26:57 2026";
        assert_eq!(parse_sysctl_sec(line), Some(1_786_742_817));
    }

    #[test]
    fn look_is_too_soon_window() {
        assert!(!look_is_too_soon(0, 1_000));
        assert!(look_is_too_soon(1_000, 1_000));
        assert!(look_is_too_soon(1_000, 1_009));
        assert!(!look_is_too_soon(1_000, 1_010));
        assert!(!look_is_too_soon(1_000, 999));
    }

    #[test]
    fn old_cursor_json_missing_last_run_is_zero() {
        let dir = std::env::temp_dir().join(format!(
            "sleep-report-old-cursor-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("cursor.json"), "{\"last_seen\":50}\n").unwrap();
        let cur = SleepCursor::load(&dir).unwrap();
        assert_eq!(cur.last_seen, 50);
        assert_eq!(cur.last_run, 0);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn repeat_look_within_ten_seconds_leaves_cursor() {
        let dir = std::env::temp_dir().join(format!(
            "sleep-report-repeat-look-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let now = now_unix();
        SleepCursor {
            last_seen: 100,
            last_run: now,
        }
        .save(&dir)
        .unwrap();
        let slept = catch_up_sleep(&dir).unwrap();
        assert!(slept.is_none());
        let cur = SleepCursor::load(&dir).unwrap();
        assert_eq!(cur.last_seen, 100);
        assert_eq!(cur.last_run, now);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn hook_json_is_object() {
        let raw = hook_additional_context("SLEEP 1m 57s");
        let v: serde_json::Value = serde_json::from_str(&raw).unwrap();
        let ctx = v["hookSpecificOutput"]["additionalContext"]
            .as_str()
            .unwrap();
        assert!(ctx.starts_with("SLEEP 1m 57s"));
        assert_eq!(
            v["hookSpecificOutput"]["hookEventName"].as_str(),
            Some("UserPromptSubmit")
        );
    }
}
