//! Blocking PR event stream. One stdout line per state change.
//!
//! Agents wrap this with the harness monitor, or exec it and wait for exit.

use std::fmt;
use std::process::{Command, Stdio};
use std::time::Duration;

/// What ends a watch after the baseline snapshot.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Until {
    /// Exit on CI green/red, review changes-requested, merged, or closed.
    Action,
    /// Exit only on merged or closed.
    Merged,
    /// Run until max-wait or signal.
    Never,
}

impl Until {
    pub fn parse(s: &str) -> Result<Self, String> {
        match s {
            "action" => Ok(Self::Action),
            "merged" => Ok(Self::Merged),
            "never" => Ok(Self::Never),
            other => Err(format!(
                "unknown --until {other:?} (want action, merged, or never)"
            )),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Target {
    pub owner: String,
    pub repo: String,
    pub number: u64,
}

impl fmt::Display for Target {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}#{}", self.owner, self.repo, self.number)
    }
}

/// Parse `owner/repo#N`, a github.com pull URL, or `N` with `--repo owner/repo`.
pub fn parse_target(spec: &str, repo_flag: Option<&str>) -> Result<Target, String> {
    if let Some(rest) = spec.strip_prefix("https://github.com/") {
        let rest = rest.trim_end_matches('/');
        let parts: Vec<&str> = rest.split('/').collect();
        if parts.len() >= 4 && parts[2] == "pull" {
            let number = parts[3]
                .parse::<u64>()
                .map_err(|_| format!("not a PR number: {}", parts[3]))?;
            return Ok(Target {
                owner: parts[0].to_string(),
                repo: parts[1].to_string(),
                number,
            });
        }
        return Err(format!("not a pull URL: {spec}"));
    }
    if let Some((left, num)) = spec.split_once('#') {
        let (owner, repo) = split_owner_repo(left)?;
        let number = num
            .parse::<u64>()
            .map_err(|_| format!("not a PR number: {num}"))?;
        return Ok(Target {
            owner,
            repo,
            number,
        });
    }
    if let Ok(number) = spec.parse::<u64>() {
        let repo = repo_flag.ok_or_else(|| "bare PR number needs --repo owner/repo".to_string())?;
        let (owner, repo) = split_owner_repo(repo)?;
        return Ok(Target {
            owner,
            repo,
            number,
        });
    }
    Err(format!(
        "want owner/repo#N, a github.com pull URL, or N with --repo (got {spec:?})"
    ))
}

fn split_owner_repo(s: &str) -> Result<(String, String), String> {
    let (owner, repo) = s
        .split_once('/')
        .ok_or_else(|| format!("want owner/repo (got {s:?})"))?;
    if owner.is_empty() || repo.is_empty() || repo.contains('/') {
        return Err(format!("want owner/repo (got {s:?})"));
    }
    Ok((owner.to_string(), repo.to_string()))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ci {
    Pending,
    Green,
    Red,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Review {
    None,
    Required,
    Approved,
    ChangesRequested,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrState {
    Open,
    Merged,
    Closed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Snapshot {
    pub state: PrState,
    pub ci: Ci,
    pub ci_failed: Option<String>,
    pub review: Review,
    pub threads: u32,
}

impl Snapshot {
    pub fn events_since(&self, prev: &Self) -> Vec<String> {
        let mut out = Vec::new();
        if self.ci != prev.ci {
            out.push(match self.ci {
                Ci::Pending => "CI PENDING".into(),
                Ci::Green => "CI GREEN".into(),
                Ci::Red => match &self.ci_failed {
                    Some(name) => format!("CI RED {name}"),
                    None => "CI RED".into(),
                },
            });
        }
        if self.review != prev.review {
            out.push(match self.review {
                Review::None => "REVIEW NONE".into(),
                Review::Required => "REVIEW REQUIRED".into(),
                Review::Approved => "REVIEW APPROVED".into(),
                Review::ChangesRequested => "REVIEW CHANGES_REQUESTED".into(),
            });
        }
        if self.threads != prev.threads {
            out.push(format!("THREADS {}", self.threads));
        }
        if self.state != prev.state {
            out.push(match self.state {
                PrState::Open => "OPEN".into(),
                PrState::Merged => "MERGED".into(),
                PrState::Closed => "CLOSED".into(),
            });
        }
        out
    }

    pub fn snapshot_lines(&self) -> Vec<String> {
        let mut lines = vec![
            match self.ci {
                Ci::Pending => "CI PENDING".into(),
                Ci::Green => "CI GREEN".into(),
                Ci::Red => match &self.ci_failed {
                    Some(name) => format!("CI RED {name}"),
                    None => "CI RED".into(),
                },
            },
            match self.review {
                Review::None => "REVIEW NONE".into(),
                Review::Required => "REVIEW REQUIRED".into(),
                Review::Approved => "REVIEW APPROVED".into(),
                Review::ChangesRequested => "REVIEW CHANGES_REQUESTED".into(),
            },
            format!("THREADS {}", self.threads),
        ];
        lines.push(match self.state {
            PrState::Open => "OPEN".into(),
            PrState::Merged => "MERGED".into(),
            PrState::Closed => "CLOSED".into(),
        });
        lines
    }

    pub fn until_hit(&self, until: Until, events: &[String]) -> Option<i32> {
        if events.iter().any(|e| e == "MERGED") {
            return Some(0);
        }
        if events.iter().any(|e| e == "CLOSED") {
            return Some(1);
        }
        match until {
            Until::Never | Until::Merged => None,
            Until::Action => {
                if events.iter().any(|e| {
                    e == "CI GREEN" || e.starts_with("CI RED") || e == "REVIEW CHANGES_REQUESTED"
                }) {
                    Some(if events.iter().any(|e| e.starts_with("CI RED")) {
                        1
                    } else {
                        0
                    })
                } else {
                    None
                }
            }
        }
    }
}

#[derive(Debug, serde::Deserialize)]
struct GhPrView {
    state: String,
    #[serde(rename = "reviewDecision", default)]
    review_decision: Option<String>,
    #[serde(rename = "statusCheckRollup", default)]
    status_check_rollup: Vec<GhCheck>,
}

#[derive(Debug, serde::Deserialize)]
struct GhCheck {
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    conclusion: Option<String>,
    #[serde(default)]
    status: Option<String>,
    #[serde(default)]
    state: Option<String>,
}

pub(crate) fn classify_ci(checks: &[GhCheck]) -> (Ci, Option<String>) {
    if checks.is_empty() {
        return (Ci::Pending, None);
    }
    let mut pending = false;
    let mut first_red = None;
    for c in checks {
        let conclusion = c.conclusion.as_deref().unwrap_or("");
        let status = c.status.as_deref().unwrap_or("");
        let state = c.state.as_deref().unwrap_or("");
        let failed = matches!(
            conclusion.to_ascii_uppercase().as_str(),
            "FAILURE" | "ERROR" | "CANCELLED" | "TIMED_OUT" | "STARTUP_FAILURE"
        ) || matches!(state.to_ascii_uppercase().as_str(), "FAILURE" | "ERROR");
        if failed {
            if first_red.is_none() {
                first_red = c
                    .name
                    .as_deref()
                    .filter(|n| !n.is_empty())
                    .map(|n| n.to_string());
            }
        } else if matches!(
            status.to_ascii_uppercase().as_str(),
            "IN_PROGRESS" | "QUEUED" | "WAITING"
        ) || matches!(state.to_ascii_uppercase().as_str(), "PENDING")
        {
            pending = true;
        }
    }
    if first_red.is_some() {
        return (Ci::Red, first_red);
    }
    if pending {
        return (Ci::Pending, None);
    }
    (Ci::Green, None)
}

pub fn classify_review(decision: Option<&str>) -> Review {
    match decision.unwrap_or("").to_ascii_uppercase().as_str() {
        "APPROVED" => Review::Approved,
        "CHANGES_REQUESTED" => Review::ChangesRequested,
        "REVIEW_REQUIRED" => Review::Required,
        _ => Review::None,
    }
}

pub fn classify_state(state: &str) -> PrState {
    match state.to_ascii_uppercase().as_str() {
        "MERGED" => PrState::Merged,
        "CLOSED" => PrState::Closed,
        _ => PrState::Open,
    }
}

pub fn snapshot_from_view(view: &str, threads: u32) -> Result<Snapshot, String> {
    let parsed: GhPrView =
        serde_json::from_str(view).map_err(|e| format!("gh pr view json: {e}"))?;
    let (ci, ci_failed) = classify_ci(&parsed.status_check_rollup);
    Ok(Snapshot {
        state: classify_state(&parsed.state),
        ci,
        ci_failed,
        review: classify_review(parsed.review_decision.as_deref()),
        threads,
    })
}

#[derive(Debug, serde::Deserialize)]
struct GqlThreads {
    data: Option<GqlData>,
}

#[derive(Debug, serde::Deserialize)]
struct GqlData {
    repository: Option<GqlRepo>,
}

#[derive(Debug, serde::Deserialize)]
struct GqlRepo {
    #[serde(rename = "pullRequest")]
    pull_request: Option<GqlPr>,
}

#[derive(Debug, serde::Deserialize)]
struct GqlPr {
    #[serde(rename = "reviewThreads")]
    review_threads: Option<GqlThreadConn>,
}

#[derive(Debug, serde::Deserialize)]
struct GqlThreadConn {
    nodes: Option<Vec<GqlThread>>,
}

#[derive(Debug, serde::Deserialize)]
struct GqlThread {
    #[serde(rename = "isResolved")]
    is_resolved: bool,
}

pub fn unresolved_thread_count(graphql_json: &str) -> Result<u32, String> {
    let parsed: GqlThreads =
        serde_json::from_str(graphql_json).map_err(|e| format!("graphql json: {e}"))?;
    let nodes = parsed
        .data
        .and_then(|d| d.repository)
        .and_then(|r| r.pull_request)
        .and_then(|p| p.review_threads)
        .and_then(|c| c.nodes)
        .unwrap_or_default();
    Ok(nodes.iter().filter(|t| !t.is_resolved).count() as u32)
}

pub fn fetch_snapshot(target: &Target) -> Result<Snapshot, String> {
    let view = gh_stdout(&[
        "pr",
        "view",
        &target.number.to_string(),
        "--repo",
        &format!("{}/{}", target.owner, target.repo),
        "--json",
        "state,reviewDecision,statusCheckRollup",
    ])?;
    let gql = gh_stdout(&[
        "api",
        "graphql",
        "-F",
        &format!("owner={}", target.owner),
        "-F",
        &format!("name={}", target.repo),
        "-F",
        &format!("number={}", target.number),
        "-f",
        "query=query($owner:String!,$name:String!,$number:Int!){repository(owner:$owner,name:$name){pullRequest(number:$number){reviewThreads(first:100){nodes{isResolved}}}}}",
    ])?;
    let threads = unresolved_thread_count(&gql)?;
    snapshot_from_view(&view, threads)
}

fn gh_stdout(args: &[&str]) -> Result<String, String> {
    let bin = std::env::var("PR_WATCH_GH").unwrap_or_else(|_| "gh".into());
    let out = Command::new(&bin)
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .map_err(|e| format!("exec {bin}: {e}"))?;
    if !out.status.success() {
        let err = String::from_utf8_lossy(&out.stderr);
        return Err(format!("{bin} {} failed: {}", args.join(" "), err.trim()));
    }
    String::from_utf8(out.stdout).map_err(|e| format!("gh stdout: {e}"))
}

pub fn sleep_interval(d: Duration) {
    std::thread::sleep(d);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_hash_spec() {
        let t = parse_target("ductone/multipass#627", None).unwrap();
        assert_eq!(t.owner, "ductone");
        assert_eq!(t.repo, "multipass");
        assert_eq!(t.number, 627);
    }

    #[test]
    fn parse_url() {
        let t = parse_target("https://github.com/ductone/c1/pull/23321/files", None).unwrap();
        assert_eq!(t.owner, "ductone");
        assert_eq!(t.repo, "c1");
        assert_eq!(t.number, 23321);
    }

    #[test]
    fn parse_bare_number_needs_repo() {
        assert!(parse_target("625", None).is_err());
        let t = parse_target("625", Some("ductone/multipass")).unwrap();
        assert_eq!(t.number, 625);
    }

    #[test]
    fn ci_red_wins_over_pending() {
        let checks = vec![
            GhCheck {
                name: Some("cli".into()),
                conclusion: Some("FAILURE".into()),
                status: Some("COMPLETED".into()),
                state: None,
            },
            GhCheck {
                name: Some("sdk".into()),
                conclusion: None,
                status: Some("IN_PROGRESS".into()),
                state: None,
            },
        ];
        let (ci, name) = classify_ci(&checks);
        assert_eq!(ci, Ci::Red);
        assert_eq!(name.as_deref(), Some("cli"));
    }

    #[test]
    fn empty_checks_are_pending() {
        assert_eq!(classify_ci(&[]).0, Ci::Pending);
    }

    #[test]
    fn changes_requested_is_its_own_event() {
        let prev = Snapshot {
            state: PrState::Open,
            ci: Ci::Pending,
            ci_failed: None,
            review: Review::Required,
            threads: 0,
        };
        let next = Snapshot {
            review: Review::ChangesRequested,
            ..prev.clone()
        };
        assert_eq!(
            next.events_since(&prev),
            vec!["REVIEW CHANGES_REQUESTED".to_string()]
        );
    }

    #[test]
    fn no_event_while_review_stays_changes_requested() {
        let snap = Snapshot {
            state: PrState::Open,
            ci: Ci::Green,
            ci_failed: None,
            review: Review::ChangesRequested,
            threads: 2,
        };
        assert!(snap.events_since(&snap).is_empty());
    }

    #[test]
    fn until_action_exits_on_changes_requested() {
        let snap = Snapshot {
            state: PrState::Open,
            ci: Ci::Pending,
            ci_failed: None,
            review: Review::ChangesRequested,
            threads: 1,
        };
        let events = vec!["REVIEW CHANGES_REQUESTED".to_string()];
        assert_eq!(snap.until_hit(Until::Action, &events), Some(0));
        assert_eq!(snap.until_hit(Until::Merged, &events), None);
    }

    #[test]
    fn until_action_exits_nonzero_on_ci_red() {
        let snap = Snapshot {
            state: PrState::Open,
            ci: Ci::Red,
            ci_failed: Some("cli".into()),
            review: Review::None,
            threads: 0,
        };
        let events = vec!["CI RED cli".to_string()];
        assert_eq!(snap.until_hit(Until::Action, &events), Some(1));
    }

    #[test]
    fn snapshot_from_view_reads_review_decision() {
        let json = r#"{
            "state":"OPEN",
            "reviewDecision":"CHANGES_REQUESTED",
            "statusCheckRollup":[{"name":"rust","conclusion":"SUCCESS","status":"COMPLETED"}]
        }"#;
        let snap = snapshot_from_view(json, 3).unwrap();
        assert_eq!(snap.review, Review::ChangesRequested);
        assert_eq!(snap.ci, Ci::Green);
        assert_eq!(snap.threads, 3);
    }

    #[test]
    fn thread_count_counts_only_unresolved() {
        let json = r#"{
            "data":{"repository":{"pullRequest":{"reviewThreads":{"nodes":[
                {"isResolved":true},{"isResolved":false},{"isResolved":false}
            ]}}}}
        }"#;
        assert_eq!(unresolved_thread_count(json).unwrap(), 2);
    }
}
