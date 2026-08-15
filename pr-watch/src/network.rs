//! Network-outage tracking for fetch failures and `--since` catch-up.

use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

use crate::sleep::fmt_dur;

/// Persisted outage window under the pr-watch state dir.
#[derive(Debug, Default, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct OutageState {
    /// Unix seconds when the current outage began. Set while fetches fail.
    #[serde(default)]
    pub open_since: Option<i64>,
    /// Start of the last completed outage of at least 5s.
    #[serde(default)]
    pub last_started: i64,
    /// End of the last completed outage of at least 5s.
    #[serde(default)]
    pub last_ended: i64,
}

impl OutageState {
    pub fn path_in(dir: &Path) -> PathBuf {
        dir.join("outage.json")
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
        let tmp = dir.join("outage.json.tmp");
        let body = serde_json::to_string_pretty(self).map_err(|e| format!("encode outage: {e}"))?;
        fs::write(&tmp, &body).map_err(|e| format!("write {}: {e}", tmp.display()))?;
        let parsed: OutageState =
            serde_json::from_str(&body).map_err(|e| format!("reparse outage: {e}"))?;
        if parsed.open_since != self.open_since
            || parsed.last_started != self.last_started
            || parsed.last_ended != self.last_ended
        {
            let _ = fs::remove_file(&tmp);
            return Err("outage write lost fields".into());
        }
        fs::rename(&tmp, &path).map_err(|e| format!("rename {}: {e}", path.display()))
    }

    /// Open an outage if one is not already open. Returns true on the transition.
    pub fn mark_down(&mut self, now: i64) -> bool {
        if self.open_since.is_some() {
            return false;
        }
        self.open_since = Some(now);
        true
    }

    /// Close an open outage. Returns the duration if it lasted at least 5s.
    pub fn recovered(&mut self, now: i64) -> Option<Duration> {
        let start = self.open_since.take()?;
        let dur = Duration::from_secs(now.saturating_sub(start).max(0) as u64);
        if dur < Duration::from_secs(5) {
            return None;
        }
        self.last_started = start;
        self.last_ended = now;
        Some(dur)
    }

    /// Last completed outage that ended after `since`.
    pub fn completed_after(&self, since: i64) -> Option<Duration> {
        if self.last_ended <= since || self.last_ended <= self.last_started {
            return None;
        }
        Some(Duration::from_secs(
            self.last_ended.saturating_sub(self.last_started) as u64,
        ))
    }
}

pub fn outage_event_line(d: Duration) -> String {
    format!("OUTAGE {}", fmt_dur(d))
}

pub fn is_network_error(err: &str) -> bool {
    let e = err.to_ascii_lowercase();
    if is_auth_error(&e) {
        return false;
    }
    NETWORK_NEEDLES.iter().any(|n| e.contains(n))
}

fn is_auth_error(e: &str) -> bool {
    e.contains("http 401")
        || e.contains("http 403")
        || e.contains("status 401")
        || e.contains("status 403")
        || e.contains("bad credentials")
        || e.contains("requires authentication")
        || e.contains("gh auth login")
}

const NETWORK_NEEDLES: &[&str] = &[
    "could not resolve host",
    "no such host",
    "nodename nor servname",
    "temporary failure in name resolution",
    "server misbehaving",
    "network is unreachable",
    "network is down",
    "host is down",
    "no route to host",
    "connection refused",
    "connection reset",
    "connection aborted",
    "broken pipe",
    "i/o timeout",
    "timeout awaiting",
    "tls handshake",
    "handshake timeout",
    "failed to connect",
    "dial tcp",
    "connect: ",
    "http 502",
    "http 503",
    "http 504",
    "502 bad gateway",
    "503 service",
    "504 gateway",
    "client.timeout exceeded",
    "eof",
    "offline",
];

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::now_unix;

    #[test]
    fn resolve_host_is_network() {
        assert!(is_network_error(
            "gh pr view 1 --repo o/r --json state failed: Get \"https://api.github.com/repos/o/r/pulls/1\": dial tcp: lookup api.github.com: no such host"
        ));
        assert!(is_network_error("could not resolve host: api.github.com"));
        assert!(is_network_error("connect: network is unreachable"));
        assert!(is_network_error("HTTP 503 Service Unavailable"));
    }

    #[test]
    fn auth_is_not_network() {
        assert!(!is_network_error("gh failed: HTTP 401 Bad credentials"));
        assert!(!is_network_error("HTTP 403: Requires authentication"));
        assert!(!is_network_error("to re-authenticate, run: gh auth login"));
    }

    #[test]
    fn parse_and_not_found_are_not_network() {
        assert!(!is_network_error(
            "gh pr view json: expected value at line 1"
        ));
        assert!(!is_network_error("HTTP 404: Not Found"));
        assert!(!is_network_error("GraphQL: API rate limit exceeded"));
    }

    #[test]
    fn mark_down_is_edge() {
        let mut s = OutageState::default();
        assert!(s.mark_down(100));
        assert!(!s.mark_down(110));
        assert_eq!(s.open_since, Some(100));
    }

    #[test]
    fn recovered_ignores_jitter() {
        let mut s = OutageState::default();
        s.mark_down(100);
        assert!(s.recovered(104).is_none());
        assert!(s.open_since.is_none());
        assert_eq!(s.last_ended, 0);
    }

    #[test]
    fn recovered_records_duration() {
        let mut s = OutageState::default();
        s.mark_down(100);
        assert_eq!(s.recovered(160), Some(Duration::from_secs(60)));
        assert_eq!(s.last_started, 100);
        assert_eq!(s.last_ended, 160);
        assert!(s.open_since.is_none());
    }

    #[test]
    fn completed_after_respects_cursor() {
        let s = OutageState {
            open_since: None,
            last_started: 100,
            last_ended: 160,
        };
        assert!(s.completed_after(160).is_none());
        assert_eq!(s.completed_after(159), Some(Duration::from_secs(60)));
        assert!(OutageState::default().completed_after(0).is_none());
    }

    #[test]
    fn event_line_uses_sleep_fmt() {
        assert_eq!(outage_event_line(Duration::from_secs(65)), "OUTAGE 1m 5s");
    }

    #[test]
    fn round_trip_file() {
        let dir = std::env::temp_dir().join(format!("pr-watch-outage-{}", now_unix()));
        let _ = fs::remove_dir_all(&dir);
        let mut s = OutageState::default();
        s.mark_down(50);
        s.save(&dir).unwrap();
        let loaded = OutageState::load(&dir).unwrap();
        assert_eq!(loaded.open_since, Some(50));
        let _ = fs::remove_dir_all(&dir);
    }
}
