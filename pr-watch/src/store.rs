//! Per cwd + repo + PR cursor under ~/.config/pr-watch/.

use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::{Snapshot, Target};

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Cursor {
    pub cwd: String,
    pub repo: String,
    pub number: u64,
    pub last_event_at: i64,
    pub snapshot: Snapshot,
}

#[derive(Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct Store {
    pub cursors: Vec<Cursor>,
}

impl Store {
    pub fn path_in(dir: &Path) -> PathBuf {
        dir.join("cursors.json")
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
        let tmp = dir.join("cursors.json.tmp");
        let body =
            serde_json::to_string_pretty(self).map_err(|e| format!("encode cursors: {e}"))?;
        fs::write(&tmp, &body).map_err(|e| format!("write {}: {e}", tmp.display()))?;
        let parsed: Store =
            serde_json::from_str(&body).map_err(|e| format!("reparse cursors: {e}"))?;
        if parsed.cursors.len() != self.cursors.len() {
            let _ = fs::remove_file(&tmp);
            return Err("cursor write lost entries".into());
        }
        fs::rename(&tmp, &path).map_err(|e| format!("rename {}: {e}", path.display()))?;
        Ok(())
    }

    fn key_pos(&self, cwd: &str, repo: &str, number: u64) -> Result<usize, usize> {
        self.cursors.binary_search_by(|c| {
            c.cwd
                .as_str()
                .cmp(cwd)
                .then_with(|| c.repo.as_str().cmp(repo))
                .then_with(|| c.number.cmp(&number))
        })
    }

    pub fn get(&self, cwd: &str, repo: &str, number: u64) -> Option<&Cursor> {
        self.key_pos(cwd, repo, number)
            .ok()
            .map(|i| &self.cursors[i])
    }

    pub fn upsert(&mut self, cursor: Cursor) {
        match self.key_pos(&cursor.cwd, &cursor.repo, cursor.number) {
            Ok(i) => self.cursors[i] = cursor,
            Err(i) => self.cursors.insert(i, cursor),
        }
    }
}

pub fn state_dir() -> PathBuf {
    if let Ok(p) = std::env::var("PR_WATCH_STATE_DIR") {
        return PathBuf::from(p);
    }
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
    PathBuf::from(home).join(".config/pr-watch")
}

pub fn canonical_cwd(explicit: Option<&str>) -> Result<String, String> {
    let raw = match explicit {
        Some(p) => PathBuf::from(p),
        None => std::env::current_dir().map_err(|e| format!("cwd: {e}"))?,
    };
    let canon = raw.canonicalize().unwrap_or(raw);
    Ok(canon.to_string_lossy().into_owned())
}

pub fn now_unix() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

/// Unix seconds, or `YYYY-MM-DDTHH:MM:SSZ`.
pub fn parse_since_stamp(s: &str) -> Result<i64, String> {
    if let Ok(n) = s.parse::<i64>() {
        if n >= 1_000_000_000 {
            return Ok(n);
        }
        return Err(format!("unix --since must be >= 1000000000 (got {n})"));
    }
    parse_rfc3339_z(s)
}

pub fn looks_like_stamp(s: &str) -> bool {
    s.contains('T')
        || (s.len() >= 10 && s.as_bytes().get(4) == Some(&b'-'))
        || s.parse::<i64>().is_ok_and(|n| n >= 1_000_000_000)
}

fn parse_rfc3339_z(s: &str) -> Result<i64, String> {
    let s = s.strip_suffix('Z').unwrap_or(s);
    let (date, time) = s
        .split_once('T')
        .ok_or_else(|| format!("want YYYY-MM-DDTHH:MM:SSZ (got {s:?})"))?;
    let dp: Vec<i64> = date.split('-').filter_map(|p| p.parse().ok()).collect();
    let tp: Vec<i64> = time.split(':').filter_map(|p| p.parse().ok()).collect();
    if dp.len() != 3 || tp.len() != 3 {
        return Err(format!("want YYYY-MM-DDTHH:MM:SSZ (got {s:?})"));
    }
    let (y, m, d) = (dp[0], dp[1], dp[2]);
    let (hh, mm, ss) = (tp[0], tp[1], tp[2]);
    if !(1..=12).contains(&m) || !(1..=31).contains(&d) || hh > 23 || mm > 59 || ss > 60 {
        return Err(format!("invalid timestamp {s:?}"));
    }
    Ok(civil_unix(y, m, d, hh, mm, ss))
}

fn civil_unix(y: i64, m: i64, d: i64, hh: i64, mm: i64, ss: i64) -> i64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = y - era * 400;
    let doy = (153 * (m + if m > 2 { -3 } else { 9 }) + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    let days = era * 146097 + doe - 719468;
    days * 86400 + hh * 3600 + mm * 60 + ss
}

/// First look (no cursor): current snapshot. Later looks: diffs vs stored snapshot.
/// `--since TIME` does not replay events already recorded at or after TIME.
pub fn catch_up_lines(
    store: &Store,
    cwd: &str,
    target: &Target,
    now: &Snapshot,
    _now_secs: i64,
    since: Option<i64>,
) -> Vec<String> {
    let repo = format!("{}/{}", target.owner, target.repo);
    match store.get(cwd, &repo, target.number) {
        None => now.snapshot_lines(),
        Some(prev) => {
            if let Some(floor) = since {
                if prev.last_event_at >= floor && prev.snapshot == *now {
                    return Vec::new();
                }
            }
            now.events_since(&prev.snapshot)
        }
    }
}

pub fn record_event(
    store: &mut Store,
    cwd: &str,
    target: &Target,
    now: Snapshot,
    now_secs: i64,
    emitted: bool,
) {
    let repo = format!("{}/{}", target.owner, target.repo);
    let last_event_at = if emitted {
        now_secs
    } else {
        store
            .get(cwd, &repo, target.number)
            .map(|c| c.last_event_at)
            .unwrap_or(now_secs)
    };
    store.upsert(Cursor {
        cwd: cwd.to_string(),
        repo,
        number: target.number,
        last_event_at,
        snapshot: now,
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Ci, PrState, Review};

    fn snap(ci: Ci, review: Review) -> Snapshot {
        Snapshot {
            state: PrState::Open,
            ci,
            ci_failed: None,
            review,
            threads: 0,
        }
    }

    #[test]
    fn insert_keeps_sorted_order() {
        let mut s = Store::default();
        let t = |cwd: &str, repo: &str, n: u64| Cursor {
            cwd: cwd.into(),
            repo: repo.into(),
            number: n,
            last_event_at: 1,
            snapshot: snap(Ci::Pending, Review::None),
        };
        s.upsert(t("/b", "o/r", 2));
        s.upsert(t("/a", "o/r", 1));
        s.upsert(t("/a", "o/r", 3));
        let keys: Vec<_> = s
            .cursors
            .iter()
            .map(|c| (c.cwd.as_str(), c.repo.as_str(), c.number))
            .collect();
        assert_eq!(
            keys,
            vec![("/a", "o/r", 1), ("/a", "o/r", 3), ("/b", "o/r", 2)]
        );
    }

    #[test]
    fn first_look_emits_snapshot() {
        let store = Store::default();
        let target = Target {
            owner: "o".into(),
            repo: "r".into(),
            number: 1,
        };
        let now = snap(Ci::Green, Review::Approved);
        let lines = catch_up_lines(&store, "/cwd", &target, &now, 10, None);
        assert!(lines.iter().any(|l| l == "CI GREEN"));
        assert!(lines.iter().any(|l| l == "REVIEW APPROVED"));
    }

    #[test]
    fn later_look_emits_only_diffs() {
        let mut store = Store::default();
        let target = Target {
            owner: "o".into(),
            repo: "r".into(),
            number: 1,
        };
        let old = snap(Ci::Pending, Review::Required);
        record_event(&mut store, "/cwd", &target, old, 10, true);
        let now = snap(Ci::Green, Review::ChangesRequested);
        let lines = catch_up_lines(&store, "/cwd", &target, &now, 20, None);
        assert_eq!(
            lines,
            vec![
                "CI GREEN".to_string(),
                "REVIEW CHANGES_REQUESTED".to_string()
            ]
        );
    }

    #[test]
    fn since_floor_skips_unchanged_after_recorded_event() {
        let mut store = Store::default();
        let target = Target {
            owner: "o".into(),
            repo: "r".into(),
            number: 1,
        };
        let now = snap(Ci::Green, Review::Approved);
        record_event(&mut store, "/cwd", &target, now.clone(), 50, true);
        let lines = catch_up_lines(&store, "/cwd", &target, &now, 60, Some(40));
        assert!(lines.is_empty());
    }

    #[test]
    fn rfc3339_z_parses() {
        assert_eq!(parse_since_stamp("1970-01-01T00:00:00Z").unwrap(), 0);
        assert_eq!(
            parse_since_stamp("2001-09-09T01:46:40Z").unwrap(),
            1_000_000_000
        );
    }

    #[test]
    fn round_trip_file() {
        let dir = std::env::temp_dir().join(format!("pr-watch-test-{}", now_unix()));
        let _ = fs::remove_dir_all(&dir);
        let mut store = Store::default();
        let target = Target {
            owner: "o".into(),
            repo: "r".into(),
            number: 9,
        };
        record_event(
            &mut store,
            "/cwd",
            &target,
            snap(Ci::Red, Review::None),
            99,
            true,
        );
        store.save(&dir).unwrap();
        let loaded = Store::load(&dir).unwrap();
        let got = loaded.get("/cwd", "o/r", 9).unwrap();
        assert_eq!(got.last_event_at, 99);
        assert_eq!(got.snapshot.ci, Ci::Red);
        let _ = fs::remove_dir_all(&dir);
    }
}
