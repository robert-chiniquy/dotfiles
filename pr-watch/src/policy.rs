//! Per-repo conclusions. Not a behavior flag: a recorded "Copilot does not
//! matter here" list, consulted by the Copilot gate.

use std::fs;
use std::path::Path;

#[derive(Debug, Default, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Policy {
    /// Sorted `owner/repo` keys for which Copilot review is not required.
    #[serde(default)]
    pub copilot_skip: Vec<String>,
}

impl Policy {
    pub fn path_in(dir: &Path) -> std::path::PathBuf {
        dir.join("policy.json")
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
        let tmp = dir.join("policy.json.tmp");
        let body = serde_json::to_string_pretty(self).map_err(|e| format!("encode policy: {e}"))?;
        fs::write(&tmp, &body).map_err(|e| format!("write {}: {e}", tmp.display()))?;
        let parsed: Policy =
            serde_json::from_str(&body).map_err(|e| format!("reparse policy: {e}"))?;
        if parsed.copilot_skip.len() != self.copilot_skip.len() {
            let _ = fs::remove_file(&tmp);
            return Err("policy write lost entries".into());
        }
        fs::rename(&tmp, &path).map_err(|e| format!("rename {}: {e}", path.display()))
    }

    pub fn copilot_required(&self, owner: &str, repo: &str) -> bool {
        let key = format!("{owner}/{repo}");
        self.copilot_skip.binary_search(&key).is_err()
    }

    pub fn skip_copilot(&mut self, owner: &str, repo: &str) {
        let key = format!("{owner}/{repo}");
        if let Err(i) = self.copilot_skip.binary_search(&key) {
            self.copilot_skip.insert(i, key);
        }
    }

    pub fn unskip_copilot(&mut self, owner: &str, repo: &str) {
        let key = format!("{owner}/{repo}");
        if let Ok(i) = self.copilot_skip.binary_search(&key) {
            self.copilot_skip.remove(i);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn skip_keeps_sorted_and_is_idempotent() {
        let mut p = Policy::default();
        p.skip_copilot("z", "r");
        p.skip_copilot("a", "r");
        p.skip_copilot("a", "r");
        assert_eq!(p.copilot_skip, vec!["a/r".to_string(), "z/r".to_string()]);
        assert!(!p.copilot_required("a", "r"));
        assert!(p.copilot_required("a", "other"));
        p.unskip_copilot("a", "r");
        assert!(p.copilot_required("a", "r"));
        assert_eq!(p.copilot_skip, vec!["z/r".to_string()]);
    }

    #[test]
    fn round_trip_file() {
        let dir = std::env::temp_dir().join(format!(
            "pr-watch-policy-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        ));
        let _ = fs::remove_dir_all(&dir);
        let mut p = Policy::default();
        p.skip_copilot("o", "r");
        p.save(&dir).unwrap();
        let loaded = Policy::load(&dir).unwrap();
        assert_eq!(loaded.copilot_skip, vec!["o/r".to_string()]);
        let _ = fs::remove_dir_all(&dir);
    }
}
