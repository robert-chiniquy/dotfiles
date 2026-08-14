//! Global YAML config. `copilot` is the default; `cwd` overrides it per
//! canonical working directory.

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

const HEADER: &str = "\
# pr-watch config
# copilot: request | skip
# cwd maps a canonical working directory to an override of copilot.
";

/// Whether this working directory must request a Copilot review before a human.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CopilotMode {
    #[default]
    Request,
    Skip,
}

impl CopilotMode {
    pub fn parse(s: &str) -> Result<Self, String> {
        match s {
            "request" => Ok(Self::Request),
            "skip" => Ok(Self::Skip),
            other => Err(format!("copilot must be request or skip (got {other:?})")),
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Request => "request",
            Self::Skip => "skip",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct CwdOverride {
    pub copilot: CopilotMode,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Config {
    #[serde(default)]
    pub copilot: CopilotMode,
    /// Canonical cwd → override. BTreeMap keeps keys sorted on insert.
    #[serde(default)]
    pub cwd: BTreeMap<String, CwdOverride>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            copilot: CopilotMode::Request,
            cwd: BTreeMap::new(),
        }
    }
}

impl Config {
    pub fn path_in(dir: &Path) -> std::path::PathBuf {
        dir.join("config.yaml")
    }

    pub fn load(dir: &Path) -> Result<Self, String> {
        let path = Self::path_in(dir);
        if !path.exists() {
            let cfg = Self::default();
            cfg.save(dir)?;
            return Ok(cfg);
        }
        let raw = fs::read_to_string(&path).map_err(|e| format!("read {}: {e}", path.display()))?;
        if raw.trim().is_empty() {
            return Err(format!("{} is empty", path.display()));
        }
        serde_yaml::from_str(&raw).map_err(|e| format!("parse {}: {e}", path.display()))
    }

    pub fn save(&self, dir: &Path) -> Result<(), String> {
        fs::create_dir_all(dir).map_err(|e| format!("mkdir {}: {e}", dir.display()))?;
        let path = Self::path_in(dir);
        let tmp = dir.join("config.yaml.tmp");
        let yaml = serde_yaml::to_string(self).map_err(|e| format!("encode config: {e}"))?;
        let body = format!("{HEADER}{yaml}");
        if body.len() < 20 {
            return Err("config encode produced an empty file".into());
        }
        fs::write(&tmp, &body).map_err(|e| format!("write {}: {e}", tmp.display()))?;
        let parsed: Config =
            serde_yaml::from_str(&body).map_err(|e| format!("reparse config: {e}"))?;
        if parsed.copilot != self.copilot || parsed.cwd.len() != self.cwd.len() {
            let _ = fs::remove_file(&tmp);
            return Err("config write lost fields".into());
        }
        fs::rename(&tmp, &path).map_err(|e| format!("rename {}: {e}", path.display()))
    }

    pub fn mode_for(&self, cwd: &str) -> CopilotMode {
        self.cwd.get(cwd).map(|o| o.copilot).unwrap_or(self.copilot)
    }

    pub fn copilot_required(&self, cwd: &str) -> bool {
        self.mode_for(cwd) == CopilotMode::Request
    }

    pub fn set_global(&mut self, mode: CopilotMode) {
        self.copilot = mode;
    }

    pub fn set_cwd(&mut self, cwd: &str, mode: CopilotMode) {
        self.cwd
            .insert(cwd.to_string(), CwdOverride { copilot: mode });
    }

    pub fn clear_cwd(&mut self, cwd: &str) {
        self.cwd.remove(cwd);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn testdir() -> std::path::PathBuf {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        std::env::temp_dir().join(format!("pr-watch-config-{nanos}"))
    }

    #[test]
    fn parse_mode_rejects_unknown() {
        assert!(CopilotMode::parse("maybe").is_err());
        assert_eq!(CopilotMode::parse("request").unwrap(), CopilotMode::Request);
        assert_eq!(CopilotMode::parse("skip").unwrap(), CopilotMode::Skip);
    }

    #[test]
    fn cwd_override_wins() {
        let mut c = Config::default();
        assert!(c.copilot_required("/a"));
        c.set_cwd("/b", CopilotMode::Skip);
        c.set_cwd("/c", CopilotMode::Request);
        assert!(c.copilot_required("/a"));
        assert!(!c.copilot_required("/b"));
        c.set_global(CopilotMode::Skip);
        assert!(!c.copilot_required("/a"));
        assert!(c.copilot_required("/c"));
        c.clear_cwd("/c");
        assert!(!c.copilot_required("/c"));
    }

    #[test]
    fn cwd_keys_stay_sorted() {
        let mut c = Config::default();
        c.set_cwd("/z", CopilotMode::Skip);
        c.set_cwd("/a", CopilotMode::Request);
        c.set_cwd("/m", CopilotMode::Skip);
        let keys: Vec<_> = c.cwd.keys().map(|s| s.as_str()).collect();
        assert_eq!(keys, vec!["/a", "/m", "/z"]);
    }

    #[test]
    fn yaml_round_trip_preserves_modes() {
        let dir = testdir();
        let _ = fs::remove_dir_all(&dir);
        let mut c = Config::default();
        c.set_global(CopilotMode::Skip);
        c.set_cwd("/work/docs", CopilotMode::Request);
        c.save(&dir).unwrap();
        let loaded = Config::load(&dir).unwrap();
        assert_eq!(loaded.copilot, CopilotMode::Skip);
        assert_eq!(loaded.mode_for("/work/docs"), CopilotMode::Request);
        assert_eq!(loaded.mode_for("/other"), CopilotMode::Skip);
        let raw = fs::read_to_string(Config::path_in(&dir)).unwrap();
        assert!(raw.contains("copilot: skip"));
        assert!(raw.contains("/work/docs"));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn yaml_with_comments_loads() {
        let raw = "\
# header
copilot: skip
cwd:
  /x:
    copilot: request
";
        let c: Config = serde_yaml::from_str(raw).unwrap();
        assert_eq!(c.copilot, CopilotMode::Skip);
        assert_eq!(c.mode_for("/x"), CopilotMode::Request);
    }

    #[test]
    fn missing_file_defaults_to_request() {
        let dir = testdir();
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let c = Config::load(&dir).unwrap();
        assert_eq!(c.copilot, CopilotMode::Request);
        assert!(Config::path_in(&dir).exists());
        let _ = fs::remove_dir_all(&dir);
    }
}
