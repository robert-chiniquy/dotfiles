//! Blocking PR event stream. One stdout line per state change.
//!
//! Agents wrap this with the harness monitor, or exec it and wait for exit.

pub mod config;
pub mod network;
pub mod sleep;
pub mod store;

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Ci {
    Pending,
    Green,
    Red,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Review {
    None,
    Required,
    Approved,
    ChangesRequested,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum PrState {
    Open,
    Merged,
    Closed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
pub enum Copilot {
    #[default]
    None,
    Requested,
    Reviewed,
    ChangesRequested,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Snapshot {
    pub state: PrState,
    pub ci: Ci,
    pub ci_failed: Option<String>,
    pub review: Review,
    pub threads: u32,
    #[serde(default)]
    pub copilot: Copilot,
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
        if self.copilot != prev.copilot {
            out.push(match self.copilot {
                Copilot::None => "COPILOT NONE".into(),
                Copilot::Requested => "COPILOT REQUESTED".into(),
                Copilot::Reviewed => "COPILOT REVIEWED".into(),
                Copilot::ChangesRequested => "COPILOT CHANGES_REQUESTED".into(),
            });
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
            match self.copilot {
                Copilot::None => "COPILOT NONE".into(),
                Copilot::Requested => "COPILOT REQUESTED".into(),
                Copilot::Reviewed => "COPILOT REVIEWED".into(),
                Copilot::ChangesRequested => "COPILOT CHANGES_REQUESTED".into(),
            },
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
                    e == "CI GREEN"
                        || e.starts_with("CI RED")
                        || e == "REVIEW CHANGES_REQUESTED"
                        || e == "COPILOT CHANGES_REQUESTED"
                        || e == "COPILOT NONE"
                        || e == "COPILOT REVIEWED"
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

/// Standing Copilot gate: emit COPILOT NONE even when that is not a
/// transition, so `--since` and watch start still force a request.
/// No-op when this repo has been recorded as Copilot-not-required.
pub fn ensure_copilot_gate(lines: &mut Vec<String>, now: &Snapshot, required: bool) {
    if !required {
        lines.retain(|l| !l.starts_with("COPILOT ") || l == "COPILOT SKIP");
        if !lines.iter().any(|l| l == "COPILOT SKIP") {
            lines.push("COPILOT SKIP".into());
        }
        return;
    }
    if now.copilot == Copilot::None && !lines.iter().any(|l| l == "COPILOT NONE") {
        lines.push("COPILOT NONE".into());
    }
}

/// Parse `owner/repo` (optional `#N` or github URL).
pub fn parse_repo_spec(spec: &str) -> Result<(String, String), String> {
    if let Some(rest) = spec.strip_prefix("https://github.com/") {
        let rest = rest.trim_end_matches('/');
        let parts: Vec<&str> = rest.split('/').collect();
        if parts.len() >= 2 {
            return split_owner_repo(&format!("{}/{}", parts[0], parts[1]));
        }
        return Err(format!("want owner/repo (got {spec:?})"));
    }
    let left = spec.split_once('#').map(|(l, _)| l).unwrap_or(spec);
    split_owner_repo(left)
}

/// One `NEXT <KIND> ...` line per distinct event kind in `events`.
/// THREADS is omitted when a changes-requested event already covers the loop.
/// REVIEW REQUIRED is omitted when a Copilot event in this batch already
/// tells the agent to request or wait.
pub fn next_actions(
    events: &[String],
    spec: &str,
    now: &Snapshot,
    copilot_required: bool,
) -> Vec<String> {
    let mut out = Vec::new();
    let mut seen = Vec::new();
    let has_changes_requested = events
        .iter()
        .any(|e| e == "REVIEW CHANGES_REQUESTED" || e == "COPILOT CHANGES_REQUESTED");
    let copilot_event = events
        .iter()
        .any(|e| e.starts_with("COPILOT ") && e != "COPILOT SKIP");
    let copilot_blocks_human =
        copilot_required && matches!(now.copilot, Copilot::None | Copilot::Requested);
    for e in events {
        let kind = event_kind(e);
        if seen.iter().any(|k| k == &kind) {
            continue;
        }
        seen.push(kind);
        if kind == "THREADS" && has_changes_requested {
            continue;
        }
        if kind == "REVIEW REQUIRED" && copilot_blocks_human && copilot_event {
            continue;
        }
        if let Some(body) = next_action_body(e, spec, now, copilot_required) {
            out.push(format!("NEXT {kind} {body}"));
        }
    }
    out
}

fn event_kind(event: &str) -> &'static str {
    if event.starts_with("CI RED") {
        "CI RED"
    } else if event.starts_with("THREADS ") {
        "THREADS"
    } else if event == "FAILED timeout" {
        "FAILED"
    } else if event.starts_with("SLEEP ") {
        "SLEEP"
    } else if event == "OUTAGE START" || event.starts_with("OUTAGE ") {
        "OUTAGE"
    } else {
        match event {
            "CI PENDING" => "CI PENDING",
            "CI GREEN" => "CI GREEN",
            "REVIEW NONE" => "REVIEW NONE",
            "REVIEW REQUIRED" => "REVIEW REQUIRED",
            "REVIEW APPROVED" => "REVIEW APPROVED",
            "REVIEW CHANGES_REQUESTED" => "REVIEW CHANGES_REQUESTED",
            "COPILOT NONE" => "COPILOT NONE",
            "COPILOT REQUESTED" => "COPILOT REQUESTED",
            "COPILOT REVIEWED" => "COPILOT REVIEWED",
            "COPILOT CHANGES_REQUESTED" => "COPILOT CHANGES_REQUESTED",
            "COPILOT SKIP" => "COPILOT SKIP",
            "OPEN" => "OPEN",
            "MERGED" => "MERGED",
            "CLOSED" => "CLOSED",
            _ => "EVENT",
        }
    }
}

fn next_action_body(
    event: &str,
    spec: &str,
    now: &Snapshot,
    copilot_required: bool,
) -> Option<String> {
    if event.starts_with("CI RED") {
        return Some(
            "read the failed check log; fix this PR's cause (do not ask to look); push; reply only if a thread named the failure; pr-watch --until action"
                .into(),
        );
    }
    if event.starts_with("SLEEP ") {
        return Some(
            "host slept during this wait; re-read current state (already fetching); do not treat the gap as a hang"
                .into(),
        );
    }
    if event == "OUTAGE START" {
        return Some(
            "GitHub fetch failed on the network; retrying; do not treat the gap as a hang or as CI"
                .into(),
        );
    }
    if event == "OUTAGE ONGOING" || event.starts_with("OUTAGE ") {
        return Some(
            "network was down; re-read current state; do not treat the gap as a hang or as CI"
                .into(),
        );
    }
    if let Some(rest) = event.strip_prefix("THREADS ") {
        if rest == "0" {
            return None;
        }
        return Some(
            "list unresolved review threads; for each: fix if reasonable else ask the user; after a pushed fix reply Addressed in <sha> and resolve; then pr-watch --until action"
                .into(),
        );
    }
    let body = match event {
        "REVIEW CHANGES_REQUESTED" | "COPILOT CHANGES_REQUESTED" => {
            if copilot_required {
                "read every unresolved thread; fix if reasonable else ask the user; after each pushed fix reply Addressed in <sha> and resolve the thread; when all such threads are done, pr-watch --until action; do not request a human until COPILOT REVIEWED"
            } else {
                "read every unresolved thread; fix if reasonable else ask the user; after each pushed fix reply Addressed in <sha> and resolve the thread; when all such threads are done, pr-watch --until action"
            }
        }
        "COPILOT SKIP" => return None,
        "COPILOT NONE" => {
            if !copilot_required {
                return None;
            }
            return Some(format!(
                "pr-watch request-copilot {spec}; do not request a human review yet; pr-watch --until action"
            ));
        }
        "COPILOT REQUESTED" => {
            if !copilot_required {
                return None;
            }
            "wait for Copilot; do not request a human review yet; pr-watch --until action"
        }
        "COPILOT REVIEWED" => {
            "address any Copilot threads first; a human review may be requested only after those are done"
        }
        "CI GREEN" => {
            return Some(if copilot_required {
                format!(
                    "if threads remain, address them; if COPILOT NONE, pr-watch request-copilot {spec}; do not request a human until COPILOT REVIEWED; do not merge a draft; do not merge unless authorized; pr-watch --since after the next push"
                )
            } else {
                "if threads remain, address them; do not merge a draft; do not merge unless authorized; pr-watch --since after the next push".into()
            });
        }
        "REVIEW APPROVED" => {
            if copilot_required {
                "if CI is not green, pr-watch --until action; if COPILOT NONE/REQUESTED, do not treat this as human-ready; if threads remain, address them; undraft is a separate decision; do not merge unless authorized"
            } else {
                "if CI is not green, pr-watch --until action; if threads remain, address them; undraft is a separate decision; do not merge unless authorized"
            }
        }
        "REVIEW REQUIRED" => {
            if !copilot_required {
                return Some(
                    "a human review may be requested; do not merge a draft; do not merge unless authorized; pr-watch --until action"
                        .into(),
                );
            }
            return Some(match now.copilot {
                Copilot::None => format!(
                    "pr-watch request-copilot {spec}; do not request a human review yet; pr-watch --until action"
                ),
                Copilot::Requested => {
                    "wait for Copilot; do not request a human review yet; pr-watch --until action"
                        .into()
                }
                Copilot::Reviewed | Copilot::ChangesRequested => {
                    "address Copilot threads first if any remain; then a human review may be requested; do not merge a draft; do not merge unless authorized; pr-watch --until action"
                        .into()
                }
            });
        }
        "MERGED" => {
            "if this was your branch and origin/main now contains the tip, prune the local worktree and the local+remote branch; do not remind about pushes"
        }
        "CLOSED" => {
            "treat as abandoned until the user says otherwise; do not reopen or recreate the branch"
        }
        "FAILED timeout" => {
            "check gh auth and network; pr-watch --since to catch anything that landed during the wait"
        }
        "CI PENDING" | "REVIEW NONE" | "OPEN" => return None,
        _ => return None,
    };
    Some(body.into())
}

#[derive(Debug, serde::Deserialize)]
struct GhPrView {
    state: String,
    #[serde(rename = "reviewDecision", default)]
    review_decision: Option<String>,
    #[serde(rename = "statusCheckRollup", default)]
    status_check_rollup: Vec<GhCheck>,
    #[serde(rename = "reviewRequests", default)]
    review_requests: Vec<GhReviewRequest>,
    #[serde(default)]
    reviews: Vec<GhReview>,
}

#[derive(Debug, serde::Deserialize)]
struct GhReviewRequest {
    #[serde(default)]
    login: Option<String>,
    #[serde(rename = "requestedReviewer", default)]
    requested_reviewer: Option<GhAuthor>,
}

impl GhReviewRequest {
    fn reviewer_login(&self) -> Option<&str> {
        self.login.as_deref().filter(|s| !s.is_empty()).or_else(|| {
            self.requested_reviewer
                .as_ref()
                .and_then(|a| a.login.as_deref())
                .filter(|s| !s.is_empty())
        })
    }
}

#[derive(Debug, serde::Deserialize)]
struct GhReview {
    #[serde(default)]
    author: Option<GhAuthor>,
    #[serde(default)]
    state: Option<String>,
}

#[derive(Debug, serde::Deserialize)]
struct GhAuthor {
    #[serde(default)]
    login: Option<String>,
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

pub fn is_copilot_login(login: &str) -> bool {
    let l = login.to_ascii_lowercase();
    let l = l.strip_suffix("[bot]").unwrap_or(l.as_str());
    l == "github-copilot" || l == "copilot" || l == "copilot-pull-request-reviewer"
}

pub(crate) fn classify_copilot(requests: &[GhReviewRequest], reviews: &[GhReview]) -> Copilot {
    let requested = requests
        .iter()
        .filter_map(GhReviewRequest::reviewer_login)
        .any(is_copilot_login);
    let mut last_state: Option<&str> = None;
    for r in reviews {
        let login = r.author.as_ref().and_then(|a| a.login.as_deref());
        if login.is_some_and(is_copilot_login) {
            last_state = r.state.as_deref();
        }
    }
    match last_state.map(|s| s.to_ascii_uppercase()) {
        Some(s) if s == "CHANGES_REQUESTED" => Copilot::ChangesRequested,
        Some(_) => Copilot::Reviewed,
        None if requested => Copilot::Requested,
        None => Copilot::None,
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
        copilot: classify_copilot(&parsed.review_requests, &parsed.reviews),
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
        "state,reviewDecision,statusCheckRollup,reviewRequests,reviews",
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

/// Request the Copilot bot as a reviewer. Idempotent enough: GitHub errors if
/// already requested; callers treat that as success.
pub fn request_copilot(target: &Target) -> Result<(), String> {
    let path = format!(
        "repos/{}/{}/pulls/{}/requested_reviewers",
        target.owner, target.repo, target.number
    );
    match gh_stdout(&[
        "api",
        "-X",
        "POST",
        &path,
        "-f",
        "reviewers[]=github-copilot",
    ]) {
        Ok(_) => Ok(()),
        Err(e) if e.to_ascii_lowercase().contains("already") => Ok(()),
        Err(e) => Err(e),
    }
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
            copilot: Copilot::None,
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
            copilot: Copilot::None,
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
            copilot: Copilot::None,
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
            copilot: Copilot::None,
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

    fn snap(copilot: Copilot) -> Snapshot {
        Snapshot {
            state: PrState::Open,
            ci: Ci::Pending,
            ci_failed: None,
            review: Review::None,
            threads: 0,
            copilot,
        }
    }

    fn na(events: &[&str], spec: &str, copilot: Copilot) -> Vec<String> {
        na_req(events, spec, copilot, true)
    }

    fn na_req(events: &[&str], spec: &str, copilot: Copilot, required: bool) -> Vec<String> {
        let ev: Vec<String> = events.iter().map(|s| (*s).to_string()).collect();
        next_actions(&ev, spec, &snap(copilot), required)
    }

    #[test]
    fn next_actions_for_changes_requested_includes_thread_loop() {
        let next = na(&["REVIEW CHANGES_REQUESTED"], "o/r#1", Copilot::None);
        assert_eq!(next.len(), 1);
        assert!(next[0].starts_with("NEXT REVIEW CHANGES_REQUESTED "));
        assert!(next[0].contains("Addressed in <sha>"));
        assert!(next[0].contains("resolve the thread"));
        assert!(next[0].contains("pr-watch --until action"));
        assert!(next[0].contains("ask the user"));
    }

    #[test]
    fn next_actions_threads_omitted_when_changes_requested() {
        let next = na(
            &["REVIEW CHANGES_REQUESTED", "THREADS 3"],
            "o/r#1",
            Copilot::None,
        );
        assert_eq!(next.len(), 1);
        assert!(next[0].starts_with("NEXT REVIEW CHANGES_REQUESTED "));
    }

    #[test]
    fn next_actions_threads_nonzero_without_verdict() {
        let next = na(&["THREADS 2"], "o/r#1", Copilot::None);
        assert_eq!(next.len(), 1);
        assert!(next[0].starts_with("NEXT THREADS "));
        assert!(next[0].contains("unresolved"));
    }

    #[test]
    fn next_actions_threads_zero_is_silent() {
        assert!(na(&["THREADS 0"], "o/r#1", Copilot::None).is_empty());
    }

    #[test]
    fn next_actions_ci_red_and_merged() {
        let next = na(&["CI RED cli", "MERGED"], "o/r#1", Copilot::None);
        assert_eq!(next.len(), 2);
        assert!(next[0].starts_with("NEXT CI RED "));
        assert!(next[0].contains("failed check"));
        assert!(next[1].starts_with("NEXT MERGED "));
        assert!(next[1].contains("prune"));
    }

    #[test]
    fn next_actions_pending_and_open_are_silent() {
        assert!(na(
            &["CI PENDING", "OPEN", "REVIEW NONE"],
            "o/r#1",
            Copilot::None
        )
        .is_empty());
    }

    #[test]
    fn copilot_login_matches_bot_names() {
        assert!(is_copilot_login("github-copilot"));
        assert!(is_copilot_login("Copilot"));
        assert!(is_copilot_login("copilot-pull-request-reviewer"));
        assert!(!is_copilot_login("github-actions"));
        assert!(!is_copilot_login("robert-chiniquy"));
    }

    #[test]
    fn classify_copilot_requested_then_reviewed() {
        let req = [GhReviewRequest {
            login: Some("github-copilot".into()),
            requested_reviewer: None,
        }];
        assert_eq!(classify_copilot(&req, &[]), Copilot::Requested);
        let reviews = [GhReview {
            author: Some(GhAuthor {
                login: Some("copilot-pull-request-reviewer".into()),
            }),
            state: Some("COMMENTED".into()),
        }];
        assert_eq!(classify_copilot(&req, &reviews), Copilot::Reviewed);
        let changes = [GhReview {
            author: Some(GhAuthor {
                login: Some("github-copilot".into()),
            }),
            state: Some("CHANGES_REQUESTED".into()),
        }];
        assert_eq!(classify_copilot(&req, &changes), Copilot::ChangesRequested);
    }

    #[test]
    fn classify_copilot_nested_requested_reviewer() {
        let req = [GhReviewRequest {
            login: None,
            requested_reviewer: Some(GhAuthor {
                login: Some("github-copilot[bot]".into()),
            }),
        }];
        assert_eq!(classify_copilot(&req, &[]), Copilot::Requested);
    }

    #[test]
    fn next_actions_copilot_none_blocks_human() {
        let next = na(
            &["COPILOT NONE", "REVIEW REQUIRED"],
            "ductone/multipass#627",
            Copilot::None,
        );
        assert_eq!(next.len(), 1);
        assert!(next[0].starts_with("NEXT COPILOT NONE "));
        assert!(next[0].contains("pr-watch request-copilot ductone/multipass#627"));
        assert!(next[0].contains("do not request a human"));
    }

    #[test]
    fn next_actions_copilot_requested_waits() {
        let next = na(&["COPILOT REQUESTED"], "o/r#1", Copilot::Requested);
        assert_eq!(next.len(), 1);
        assert!(next[0].contains("do not request a human"));
        assert!(next[0].contains("wait for Copilot"));
    }

    #[test]
    fn next_actions_review_required_alone_requests_copilot() {
        let next = na(&["REVIEW REQUIRED"], "o/r#1", Copilot::None);
        assert_eq!(next.len(), 1);
        assert!(next[0].starts_with("NEXT REVIEW REQUIRED "));
        assert!(next[0].contains("pr-watch request-copilot o/r#1"));
        assert!(next[0].contains("do not request a human"));
    }

    #[test]
    fn next_actions_review_required_after_copilot_allows_human() {
        let next = na(&["REVIEW REQUIRED"], "o/r#1", Copilot::Reviewed);
        assert_eq!(next.len(), 1);
        assert!(next[0].contains("human review may be requested"));
    }

    #[test]
    fn ensure_copilot_gate_appends_once() {
        let now = snap(Copilot::None);
        let mut lines = vec!["CI GREEN".into()];
        ensure_copilot_gate(&mut lines, &now, true);
        ensure_copilot_gate(&mut lines, &now, true);
        assert_eq!(
            lines,
            vec!["CI GREEN".to_string(), "COPILOT NONE".to_string()]
        );
        let reviewed = snap(Copilot::Reviewed);
        let mut already = vec!["CI GREEN".into()];
        ensure_copilot_gate(&mut already, &reviewed, true);
        assert_eq!(already, vec!["CI GREEN".to_string()]);
    }

    #[test]
    fn until_action_exits_on_copilot_none_and_reviewed() {
        let none = snap(Copilot::None);
        assert_eq!(
            none.until_hit(Until::Action, &["COPILOT NONE".into()]),
            Some(0)
        );
        assert_eq!(
            none.until_hit(Until::Action, &["COPILOT REVIEWED".into()]),
            Some(0)
        );
        assert_eq!(
            none.until_hit(Until::Action, &["COPILOT CHANGES_REQUESTED".into()]),
            Some(0)
        );
        assert_eq!(
            none.until_hit(Until::Action, &["COPILOT REQUESTED".into()]),
            None
        );
    }

    #[test]
    fn snapshot_from_view_reads_copilot_request() {
        let json = r#"{
            "state":"OPEN",
            "reviewDecision":"REVIEW_REQUIRED",
            "statusCheckRollup":[],
            "reviewRequests":[{"login":"github-copilot"}],
            "reviews":[]
        }"#;
        let snap = snapshot_from_view(json, 0).unwrap();
        assert_eq!(snap.copilot, Copilot::Requested);
        assert_eq!(snap.review, Review::Required);
    }

    #[test]
    fn copilot_bot_suffix_is_copilot() {
        assert!(is_copilot_login("github-copilot[bot]"));
        assert!(!is_copilot_login("not-a-copilot-user"));
    }

    #[test]
    fn next_actions_skip_allows_human() {
        let next = na_req(&["REVIEW REQUIRED"], "o/r#1", Copilot::None, false);
        assert_eq!(next.len(), 1);
        assert!(next[0].contains("human review may be requested"));
        assert!(!next[0].contains("request-copilot"));
    }

    #[test]
    fn ensure_copilot_gate_skip_replaces_none() {
        let now = snap(Copilot::None);
        let mut lines = vec!["CI GREEN".into(), "COPILOT NONE".into()];
        ensure_copilot_gate(&mut lines, &now, false);
        assert!(lines.iter().any(|l| l == "COPILOT SKIP"));
        assert!(!lines.iter().any(|l| l == "COPILOT NONE"));
    }

    #[test]
    fn parse_repo_spec_strips_pr() {
        let (o, r) = parse_repo_spec("ductone/c1#23321").unwrap();
        assert_eq!((o, r), ("ductone".into(), "c1".into()));
    }

    #[test]
    fn next_actions_sleep() {
        let next = na(&["SLEEP 12m 4s"], "o/r#1", Copilot::None);
        assert_eq!(next.len(), 1);
        assert!(next[0].starts_with("NEXT SLEEP "));
        assert!(next[0].contains("host slept"));
    }

    #[test]
    fn next_actions_outage_completed() {
        let next = na(&["OUTAGE 3m"], "o/r#1", Copilot::None);
        assert_eq!(next.len(), 1);
        assert!(next[0].starts_with("NEXT OUTAGE "));
        assert!(next[0].contains("network was down"));
        assert!(next[0].contains("not treat the gap as a hang"));
    }

    #[test]
    fn next_actions_outage_start() {
        let next = na(&["OUTAGE START"], "o/r#1", Copilot::None);
        assert_eq!(next.len(), 1);
        assert!(next[0].starts_with("NEXT OUTAGE "));
        assert!(next[0].contains("retrying"));
    }

    #[test]
    fn until_action_does_not_exit_on_outage() {
        let snap = snap(Copilot::None);
        assert_eq!(
            snap.until_hit(Until::Action, &["OUTAGE START".into()]),
            None
        );
        assert_eq!(snap.until_hit(Until::Action, &["OUTAGE 1m".into()]), None);
    }
}
