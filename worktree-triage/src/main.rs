//! worktree-triage — report on and safely reclaim disk space from git
//! worktrees living under a repo root (default `~/repo`).
//! Std-only: shells out to `git`, `du`, and (optionally) `gh`; no external
//! crates, including no JSON library — `gh`'s `--json` output is parsed by a
//! small hand-rolled scanner tailored to the flat `number`/`headRefName`/
//! `state` fields this tool asks for, not a general JSON parser.
//!
//! Usage:
//!   worktree-triage                  # read-only report (default, never mutates)
//!   worktree-triage --clean-artifacts [--dry-run]
//!                                     # delete regenerable build dirs (target/
//!                                     # build/node_modules/dist) from every
//!                                     # worktree; never removes a worktree
//!   worktree-triage --prune-safe [--dry-run]
//!                                     # `git worktree remove` on SAFE-PRUNE
//!                                     # worktrees only
//!   worktree-triage --root PATH      # scan a different repo root
//!   worktree-triage --no-gh          # skip GitHub PR lookups, local-only
//!
//! Discovery: each immediate subdirectory of the root whose `.git` is a
//! directory (a real checkout, not a linked worktree) is treated as a repo.
//! Its worktrees, including ones registered as sibling directories and not
//! only ones nested under `.worktrees/`, are enumerated via
//! `git worktree list --porcelain`, which reports absolute paths regardless
//! of where they physically live.
//!
//! Status resolution, most authoritative first:
//!   PR-MERGED  the branch has a merged GitHub PR (catches squash-merges
//!              that local git ancestry can't see).
//!   PR-OPEN    the branch has an open GitHub PR — always REVIEW, even if
//!              local git looks merged: it's active work.
//!   PR-CLOSED  the branch has a closed-not-merged PR — abandoned, REVIEW.
//!   MERGED     (no PR data: not on GitHub, no PR, detached HEAD, or `gh`
//!              unavailable) the worktree's HEAD is an ancestor of main.
//!   ABSORBED   as above, not MERGED, but `main...HEAD` has an empty diff
//!              (covers squash-merges when there's no PR to ask about).
//!   UNMERGED   otherwise.
//! `gh pr list` is called once per repo (not once per worktree/branch) and
//! matched locally against every worktree's branch name. No per-call
//! timeout is enforced, so a hung `gh` call blocks the scan for that repo;
//! a failed or missing `gh` degrades that repo to local-only classification
//! rather than failing the whole report.
//!
//! Safety classification (conservative: a false "safe" causes data loss):
//!   SAFE-PRUNE = (PR-MERGED, MERGED, or ABSORBED) and the worktree has no
//!                uncommitted or untracked changes. Everything else,
//!                including PR-OPEN and PR-CLOSED regardless of local git
//!                state, is REVIEW. The repo's own main worktree is always
//!                REVIEW (never a `--prune-safe` target) regardless of
//!                merge status.
//!
//! Build artifacts (target/, build/, node_modules/, dist/, src-tauri/target/)
//! are regenerable and reported/cleanable on every worktree, safe or not.

use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{exit, Command};

const DEFAULT_ROOT: &str = "/Users/rch/repo";
const ARTIFACT_DIRS: &[&str] = &["target", "build", "node_modules", "dist", "src-tauri/target"];

// ---------------------------------------------------------------------
// types
// ---------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MergeStatus {
    Merged,
    Absorbed,
    Unmerged,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Classification {
    SafePrune,
    Review,
}

impl Classification {
    fn label(self) -> &'static str {
        match self {
            Classification::SafePrune => "SAFE-PRUNE",
            Classification::Review => "REVIEW",
        }
    }
}

/// GitHub PR state, as reported by `gh pr list --json state`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PrState {
    Merged,
    Open,
    Closed,
}

impl PrState {
    fn label(self) -> &'static str {
        match self {
            PrState::Merged => "MERGED",
            PrState::Open => "OPEN",
            PrState::Closed => "CLOSED",
        }
    }
}

/// One row of `gh pr list --json number,headRefName,state,mergedAt`.
#[derive(Debug, Clone, PartialEq, Eq)]
struct PrRecord {
    number: u64,
    head_ref_name: String,
    state: PrState,
}

/// A worktree's resolved status, PR data taking priority over local git
/// ancestry when a PR for the branch is known (see module docs).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ResolvedStatus {
    PrMerged,
    PrOpen,
    PrClosed,
    Merged,
    Absorbed,
    Unmerged,
}

impl ResolvedStatus {
    fn label(self) -> &'static str {
        match self {
            ResolvedStatus::PrMerged => "PR-MERGED",
            ResolvedStatus::PrOpen => "PR-OPEN",
            ResolvedStatus::PrClosed => "PR-CLOSED",
            ResolvedStatus::Merged => "MERGED",
            ResolvedStatus::Absorbed => "ABSORBED",
            ResolvedStatus::Unmerged => "UNMERGED",
        }
    }

    fn from_local(status: MergeStatus) -> ResolvedStatus {
        match status {
            MergeStatus::Merged => ResolvedStatus::Merged,
            MergeStatus::Absorbed => ResolvedStatus::Absorbed,
            MergeStatus::Unmerged => ResolvedStatus::Unmerged,
        }
    }
}

/// One entry from `git status --porcelain=v1`: the two-character XY status
/// code and the path (for renames, `old -> new`).
#[derive(Debug, Clone, PartialEq, Eq)]
struct StatusEntry {
    code: String,
    path: String,
}

/// One entry from `git worktree list --porcelain`.
#[derive(Debug, Clone, PartialEq, Eq)]
struct WorktreeRecord {
    path: PathBuf,
    head: String,
    branch: Option<String>,
}

struct WorktreeInfo {
    repo: String,
    path: PathBuf,
    branch: String,
    status: ResolvedStatus,
    pr: Option<(PrState, u64)>,
    dirty_entries: Vec<StatusEntry>,
    total_kb: u64,
    artifact_kb: u64,
    classification: Classification,
    is_main: bool,
}

// ---------------------------------------------------------------------
// pure functions
// ---------------------------------------------------------------------

/// Classify merge status from two git-command outcomes:
/// `is_ancestor` = `git merge-base --is-ancestor <head> <main>` succeeded.
/// `diff_empty`  = `git diff --quiet <main>...<head>` succeeded (only
/// meaningful, and only checked, when `is_ancestor` is false).
fn classify_status(is_ancestor: bool, diff_empty: bool) -> MergeStatus {
    if is_ancestor {
        MergeStatus::Merged
    } else if diff_empty {
        MergeStatus::Absorbed
    } else {
        MergeStatus::Unmerged
    }
}

/// Map a GitHub PR state to the resolved status it implies. Returns `None`
/// when there's no PR for this branch (no PR, detached HEAD, or `gh`
/// unavailable) — the caller falls back to local git classification.
fn pr_state_to_status(pr: Option<PrState>) -> Option<ResolvedStatus> {
    match pr {
        Some(PrState::Merged) => Some(ResolvedStatus::PrMerged),
        Some(PrState::Open) => Some(ResolvedStatus::PrOpen),
        Some(PrState::Closed) => Some(ResolvedStatus::PrClosed),
        None => None,
    }
}

/// Resolve a worktree's status: PR data wins when present, else fall back
/// to local git ancestry (`classify_status`).
fn resolve_status(pr: Option<PrState>, is_ancestor: bool, diff_empty: bool) -> ResolvedStatus {
    pr_state_to_status(pr).unwrap_or_else(|| ResolvedStatus::from_local(classify_status(is_ancestor, diff_empty)))
}

/// SAFE-PRUNE iff the resolved status is absorbed-by-something (a merged PR,
/// local ancestry, or an empty diff) and there are no uncommitted or
/// untracked changes. PR-OPEN and PR-CLOSED are always REVIEW, regardless
/// of what local git ancestry alone would suggest.
fn classify(status: ResolvedStatus, dirty: usize) -> Classification {
    match status {
        ResolvedStatus::PrOpen | ResolvedStatus::PrClosed | ResolvedStatus::Unmerged => {
            Classification::Review
        }
        _ if dirty > 0 => Classification::Review,
        _ => Classification::SafePrune,
    }
}

/// Parse the branch name out of `git symbolic-ref refs/remotes/origin/HEAD`
/// output, e.g. `refs/remotes/origin/main` -> `Some("main")`.
fn parse_origin_head(output: &str) -> Option<String> {
    let trimmed = output.trim();
    trimmed
        .strip_prefix("refs/remotes/origin/")
        .map(|s| s.to_string())
        .filter(|s| !s.is_empty())
}

/// Format a `du -sk`-style kilobyte count as a human-readable size.
fn human_size(kb: u64) -> String {
    const KB: f64 = 1024.0;
    let bytes = kb as f64 * 1024.0;
    if kb < 1024 {
        format!("{kb}KB")
    } else if kb < 1024 * 1024 {
        format!("{:.1}MB", bytes / (KB * KB))
    } else {
        format!("{:.2}GB", bytes / (KB * KB * KB))
    }
}

/// Parse `git worktree list --porcelain` output into records. Blocks are
/// separated by blank lines; each has a `worktree`, `HEAD`, and either
/// `branch refs/heads/<name>`, `detached`, or `bare`.
fn parse_worktree_porcelain(input: &str) -> Vec<WorktreeRecord> {
    let mut records = Vec::new();
    let mut path: Option<PathBuf> = None;
    let mut head: Option<String> = None;
    let mut branch: Option<String> = None;
    let mut is_bare = false;

    let flush = |path: &mut Option<PathBuf>,
                      head: &mut Option<String>,
                      branch: &mut Option<String>,
                      is_bare: &mut bool,
                      records: &mut Vec<WorktreeRecord>| {
        if let (Some(p), Some(h)) = (path.take(), head.take()) {
            if !*is_bare {
                records.push(WorktreeRecord {
                    path: p,
                    head: h,
                    branch: branch.take(),
                });
            }
        }
        branch.take();
        *is_bare = false;
    };

    for line in input.lines() {
        if line.is_empty() {
            flush(&mut path, &mut head, &mut branch, &mut is_bare, &mut records);
            continue;
        }
        if let Some(rest) = line.strip_prefix("worktree ") {
            path = Some(PathBuf::from(rest));
        } else if let Some(rest) = line.strip_prefix("HEAD ") {
            head = Some(rest.to_string());
        } else if let Some(rest) = line.strip_prefix("branch ") {
            branch = rest
                .strip_prefix("refs/heads/")
                .map(|s| s.to_string())
                .or_else(|| Some(rest.to_string()));
        } else if line == "bare" {
            is_bare = true;
        }
        // "detached", "locked", "prunable" and any future annotation lines
        // are ignored: absence of `branch` already means detached.
    }
    flush(&mut path, &mut head, &mut branch, &mut is_bare, &mut records);
    records
}

/// Sum `du -sk` KB values for whichever of ARTIFACT_DIRS exist under `root`.
fn artifact_kb_for(root: &Path, du_kb: impl Fn(&Path) -> Option<u64>) -> u64 {
    ARTIFACT_DIRS
        .iter()
        .map(|rel| root.join(rel))
        .filter(|p| p.is_dir())
        .filter_map(|p| du_kb(&p))
        .sum()
}

/// Parse `git status --porcelain=v1` output into (code, path) entries. A
/// rename line (`R  old -> new`) keeps the arrow in its path so the user
/// sees both sides.
fn parse_status_porcelain(input: &str) -> Vec<StatusEntry> {
    input
        .lines()
        .filter(|l| l.len() > 3)
        .map(|l| StatusEntry {
            code: l[..2].to_string(),
            path: l[3..].to_string(),
        })
        .collect()
}

/// Categorize one status entry into a short label for the dirty breakdown:
/// `??` untracked, `R` renamed, `D` deleted (either side), `M` modified
/// (either side), `A` any other staged change.
fn categorize_status_entry(entry: &StatusEntry) -> &'static str {
    let mut chars = entry.code.chars();
    let x = chars.next().unwrap_or(' ');
    let y = chars.next().unwrap_or(' ');
    if entry.code == "??" {
        "??"
    } else if x == 'R' || y == 'R' {
        "R"
    } else if x == 'D' || y == 'D' {
        "D"
    } else if x == 'M' || y == 'M' {
        "M"
    } else {
        "A"
    }
}

/// Count entries by category (M/A/D/R/??), in the same stable order used by
/// `dirty_breakdown`. Shared by `dirty_breakdown` (all entries) and
/// `summarize_overflow` (entries beyond the shown cap).
fn category_counts(entries: &[StatusEntry]) -> Vec<(&'static str, usize)> {
    let mut counts: Vec<(&'static str, usize)> = Vec::new();
    for entry in entries {
        let kind = categorize_status_entry(entry);
        match counts.iter_mut().find(|(k, _)| *k == kind) {
            Some((_, n)) => *n += 1,
            None => counts.push((kind, 1)),
        }
    }
    let order = ["M", "A", "D", "R", "??"];
    counts.sort_by_key(|(k, _)| order.iter().position(|o| o == k).unwrap_or(order.len()));
    counts
}

/// Compact dirty-state summary for the table's DIRTY column, e.g. `2M 1??`,
/// or `clean` when there are no entries.
fn dirty_breakdown(entries: &[StatusEntry]) -> String {
    if entries.is_empty() {
        return "clean".to_string();
    }
    category_counts(entries)
        .into_iter()
        .map(|(k, n)| format!("{n}{k}"))
        .collect::<Vec<_>>()
        .join(" ")
}

/// Group digits with commas: `1182` -> `1,182`.
fn with_thousands_separator(n: usize) -> String {
    let digits = n.to_string();
    let mut out = String::new();
    for (i, c) in digits.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            out.push(',');
        }
        out.push(c);
    }
    out.chars().rev().collect()
}

/// Rollup line for the entries beyond `shown` in a REVIEW DETAILS listing,
/// e.g. `... and 1,182 more (D:1180 M:1 ??:1)`. Empty string when there's
/// nothing beyond `shown` (nothing to roll up, no line printed).
fn summarize_overflow(entries: &[StatusEntry], shown: usize) -> String {
    if entries.len() <= shown {
        return String::new();
    }
    let overflow = &entries[shown..];
    let breakdown = category_counts(overflow)
        .into_iter()
        .map(|(k, n)| format!("{k}:{n}"))
        .collect::<Vec<_>>()
        .join(" ");
    format!(
        "... and {} more ({breakdown})",
        with_thousands_separator(overflow.len())
    )
}

/// Split a `gh --json` array's top-level objects out of the raw output,
/// tracking string/escape state so braces inside string values don't
/// confuse the brace-depth count. Not a general JSON parser: it only needs
/// to find object boundaries, not validate structure.
fn split_json_objects(json: &str) -> Vec<&str> {
    let bytes = json.as_bytes();
    let mut objects = Vec::new();
    let mut in_string = false;
    let mut escape = false;
    let mut depth = 0i32;
    let mut start = None;
    for (i, &b) in bytes.iter().enumerate() {
        let c = b as char;
        if in_string {
            if escape {
                escape = false;
            } else if c == '\\' {
                escape = true;
            } else if c == '"' {
                in_string = false;
            }
            continue;
        }
        match c {
            '"' => in_string = true,
            '{' => {
                if depth == 0 {
                    start = Some(i);
                }
                depth += 1;
            }
            '}' => {
                depth -= 1;
                if depth == 0 {
                    if let Some(s) = start {
                        objects.push(&json[s..=i]);
                    }
                    start = None;
                }
            }
            _ => {}
        }
    }
    objects
}

/// Extract a `"key":"value"` string field from a JSON object substring,
/// unescaping `\"`, `\\`, `\n`, `\t`. Does not handle unicode escapes;
/// `gh`'s branch names and PR states never need them.
fn json_string_field(obj: &str, key: &str) -> Option<String> {
    let needle = format!("\"{key}\":\"");
    let start = obj.find(&needle)? + needle.len();
    let mut out = String::new();
    let mut escape = false;
    for c in obj[start..].chars() {
        if escape {
            out.push(match c {
                'n' => '\n',
                't' => '\t',
                other => other,
            });
            escape = false;
            continue;
        }
        if c == '\\' {
            escape = true;
            continue;
        }
        if c == '"' {
            return Some(out);
        }
        out.push(c);
    }
    None
}

/// Extract a `"key":<digits>` numeric field from a JSON object substring.
fn json_number_field(obj: &str, key: &str) -> Option<u64> {
    let needle = format!("\"{key}\":");
    let start = obj.find(&needle)? + needle.len();
    let digits: String = obj[start..]
        .trim_start()
        .chars()
        .take_while(|c| c.is_ascii_digit())
        .collect();
    digits.parse().ok()
}

/// Parse `gh pr list --json number,headRefName,state,mergedAt` output.
/// Objects whose `state` isn't one of MERGED/OPEN/CLOSED, or that are
/// missing a required field, are skipped rather than failing the whole
/// parse — a partial PR map degrades gracefully to local classification
/// for the branches it's missing.
fn parse_pr_list_json(json: &str) -> Vec<PrRecord> {
    split_json_objects(json)
        .into_iter()
        .filter_map(|obj| {
            let number = json_number_field(obj, "number")?;
            let head_ref_name = json_string_field(obj, "headRefName")?;
            let state = match json_string_field(obj, "state")?.as_str() {
                "MERGED" => PrState::Merged,
                "OPEN" => PrState::Open,
                "CLOSED" => PrState::Closed,
                _ => return None,
            };
            Some(PrRecord {
                number,
                head_ref_name,
                state,
            })
        })
        .collect()
}

/// Extract an `owner/repo` slug from a git remote URL, handling both the
/// `git@host:owner/repo.git` (SSH) and `https://host/owner/repo.git` forms.
fn repo_slug_from_origin_url(url: &str) -> Option<String> {
    let trimmed = url.trim();
    let without_git = trimmed.strip_suffix(".git").unwrap_or(trimmed);
    let path = if let Some(rest) = without_git.strip_prefix("git@") {
        rest.split_once(':').map(|(_, p)| p)?
    } else if let Some(rest) = without_git.strip_prefix("ssh://") {
        rest.split_once('/')?.1
    } else if let Some(rest) = without_git.strip_prefix("https://") {
        rest.split_once('/')?.1
    } else if let Some(rest) = without_git.strip_prefix("http://") {
        rest.split_once('/')?.1
    } else {
        return None;
    };
    let segments: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
    if segments.len() < 2 {
        return None;
    }
    let n = segments.len();
    Some(format!("{}/{}", segments[n - 2], segments[n - 1]))
}

// ---------------------------------------------------------------------
// git / du plumbing
// ---------------------------------------------------------------------

fn run(cwd: Option<&Path>, args: &[&str]) -> Option<(bool, String)> {
    let mut cmd = Command::new(args[0]);
    cmd.args(&args[1..]);
    if let Some(dir) = cwd {
        cmd.current_dir(dir);
    }
    let output = cmd.output().ok()?;
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    Some((output.status.success(), stdout))
}

fn git_ok(cwd: &Path, args: &[&str]) -> bool {
    let mut full = vec!["git"];
    full.extend_from_slice(args);
    run(Some(cwd), &full).map(|(ok, _)| ok).unwrap_or(false)
}

fn git_out(cwd: &Path, args: &[&str]) -> Option<String> {
    let mut full = vec!["git"];
    full.extend_from_slice(args);
    run(Some(cwd), &full).and_then(|(ok, out)| ok.then_some(out))
}

fn du_kb(path: &Path) -> Option<u64> {
    let (ok, out) = run(None, &["du", "-sk", path.to_str()?])?;
    if !ok {
        return None;
    }
    out.split_whitespace().next()?.parse().ok()
}

fn gh_available() -> bool {
    run(None, &["which", "gh"]).map(|(ok, _)| ok).unwrap_or(false)
}

/// One `gh pr list` call per repo, batched across all its branches. Returns
/// `Err` with a human-readable reason on any failure (not installed, not
/// authed, not a GitHub remote, rate-limited, etc.) so the caller can print
/// one notice and fall back to local-only classification for this repo.
fn fetch_pr_map(slug: &str) -> Result<HashMap<String, PrRecord>, String> {
    let (ok, out) = run(
        None,
        &[
            "gh",
            "pr",
            "list",
            "--repo",
            slug,
            "--state",
            "all",
            "--limit",
            "500",
            "--json",
            "number,headRefName,state,mergedAt",
        ],
    )
    .ok_or_else(|| format!("failed to run gh for {slug}"))?;
    if !ok {
        return Err(format!("gh pr list failed for {slug}"));
    }
    let mut map = HashMap::new();
    for record in parse_pr_list_json(&out) {
        map.insert(record.head_ref_name.clone(), record);
    }
    Ok(map)
}

fn is_git_repo_root(dir: &Path) -> bool {
    dir.join(".git").is_dir()
}

fn discover_repos(root: &Path) -> Vec<PathBuf> {
    let mut repos = Vec::new();
    let Ok(entries) = fs::read_dir(root) else {
        return repos;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() && is_git_repo_root(&path) {
            repos.push(path);
        }
    }
    repos.sort();
    repos
}

/// Resolve the repo's main branch name: origin/HEAD symbolic ref, else
/// whichever of `main`/`master` exists as a local branch.
fn resolve_main_branch(repo: &Path) -> Option<String> {
    if let Some(out) = git_out(repo, &["symbolic-ref", "--quiet", "refs/remotes/origin/HEAD"]) {
        if let Some(branch) = parse_origin_head(&out) {
            return Some(branch);
        }
    }
    for candidate in ["main", "master"] {
        if git_ok(
            repo,
            &["show-ref", "--verify", "--quiet", &format!("refs/heads/{candidate}")],
        ) {
            return Some(candidate.to_string());
        }
    }
    None
}

fn list_worktrees(repo: &Path) -> Vec<WorktreeRecord> {
    match git_out(repo, &["worktree", "list", "--porcelain"]) {
        Some(out) => parse_worktree_porcelain(&out),
        None => Vec::new(),
    }
}

fn origin_url(repo: &Path) -> Option<String> {
    git_out(repo, &["remote", "get-url", "origin"]).map(|s| s.trim().to_string())
}

fn build_worktree_info(
    repo: &Path,
    repo_name: &str,
    main_branch: &str,
    pr_map: &HashMap<String, PrRecord>,
    rec: &WorktreeRecord,
) -> Option<WorktreeInfo> {
    if !rec.path.is_dir() {
        return None;
    }
    let dirty_entries = git_out(&rec.path, &["status", "--porcelain=v1"])
        .map(|out| parse_status_porcelain(&out))
        .unwrap_or_default();

    let pr = rec
        .branch
        .as_ref()
        .and_then(|b| pr_map.get(b))
        .map(|r| (r.state, r.number));

    let is_ancestor = git_ok(repo, &["merge-base", "--is-ancestor", &rec.head, main_branch]);
    let diff_empty = is_ancestor
        || git_ok(repo, &["diff", "--quiet", &format!("{main_branch}...{}", rec.head)]);
    let status = resolve_status(pr.map(|(state, _)| state), is_ancestor, diff_empty);

    let is_main = same_path(&rec.path, repo);
    let mut classification = classify(status, dirty_entries.len());
    if is_main {
        classification = Classification::Review;
    }

    let total_kb = du_kb(&rec.path).unwrap_or(0);
    let artifact_kb = artifact_kb_for(&rec.path, du_kb);

    Some(WorktreeInfo {
        repo: repo_name.to_string(),
        path: rec.path.clone(),
        branch: rec.branch.clone().unwrap_or_else(|| "detached".to_string()),
        status,
        pr,
        dirty_entries,
        total_kb,
        artifact_kb,
        classification,
        is_main,
    })
}

fn same_path(a: &Path, b: &Path) -> bool {
    fs::canonicalize(a).ok() == fs::canonicalize(b).ok()
}

// ---------------------------------------------------------------------
// CLI
// ---------------------------------------------------------------------

struct Args {
    root: PathBuf,
    clean_artifacts: bool,
    prune_safe: bool,
    dry_run: bool,
    no_gh: bool,
}

fn parse_args(argv: &[String]) -> Result<Args, String> {
    let mut root = PathBuf::from(DEFAULT_ROOT);
    let mut clean_artifacts = false;
    let mut prune_safe = false;
    let mut dry_run = false;
    let mut no_gh = false;

    let mut i = 0;
    while i < argv.len() {
        match argv[i].as_str() {
            "--root" => {
                i += 1;
                let val = argv.get(i).ok_or("--root requires a path")?;
                root = PathBuf::from(val);
            }
            "--clean-artifacts" => clean_artifacts = true,
            "--prune-safe" => prune_safe = true,
            "--dry-run" => dry_run = true,
            "--no-gh" => no_gh = true,
            "-h" | "--help" => return Err(HELP.to_string()),
            other => return Err(format!("unknown argument: {other}")),
        }
        i += 1;
    }

    Ok(Args {
        root,
        clean_artifacts,
        prune_safe,
        dry_run,
        no_gh,
    })
}

const HELP: &str = "\
worktree-triage [--root PATH] [--clean-artifacts] [--prune-safe] [--dry-run] [--no-gh]

  (no flags)          read-only report, never mutates anything
  --clean-artifacts   delete target/build/node_modules/dist from every worktree
  --prune-safe        git worktree remove on SAFE-PRUNE worktrees only
  --dry-run           combined with an action flag, print without doing it
  --root PATH         scan a different repo root (default /Users/rch/repo)
  --no-gh             skip GitHub PR lookups, classify from local git only";

// ---------------------------------------------------------------------
// main
// ---------------------------------------------------------------------

fn gather(root: &Path, use_gh: bool) -> Vec<WorktreeInfo> {
    let use_gh = if use_gh && !gh_available() {
        eprintln!("worktree-triage: gh not found; running local-only classification");
        false
    } else {
        use_gh
    };
    let mut infos = Vec::new();
    for repo in discover_repos(root) {
        let repo_name = repo
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();
        let Some(main_branch) = resolve_main_branch(&repo) else {
            continue;
        };

        let pr_map = if use_gh {
            match origin_url(&repo).as_deref().and_then(repo_slug_from_origin_url) {
                Some(slug) => match fetch_pr_map(&slug) {
                    Ok(map) => map,
                    Err(e) => {
                        eprintln!("worktree-triage: {e}; using local-only classification for {repo_name}");
                        HashMap::new()
                    }
                },
                None => HashMap::new(),
            }
        } else {
            HashMap::new()
        };

        for rec in list_worktrees(&repo) {
            if let Some(info) = build_worktree_info(&repo, &repo_name, &main_branch, &pr_map, &rec) {
                infos.push(info);
            }
        }
    }
    infos.sort_by(|a, b| b.artifact_kb.cmp(&a.artifact_kb));
    infos
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}...", &s[..max.saturating_sub(3)])
    }
}

/// Compact PR column string, e.g. `MERGED #22954`, `OPEN #22870`, or `-`
/// when there's no known PR for the branch.
fn pr_column(pr: Option<(PrState, u64)>) -> String {
    match pr {
        Some((state, number)) => format!("{} #{number}", state.label()),
        None => "-".to_string(),
    }
}

/// True when a worktree's status would make it prune-eligible if only it
/// were clean — the "merged but dirty" case worth calling out specifically,
/// as distinct from a worktree that's dirty AND still unmerged/under review.
fn would_be_safe_if_clean(info: &WorktreeInfo) -> bool {
    !info.is_main
        && matches!(
            info.status,
            ResolvedStatus::PrMerged | ResolvedStatus::Merged | ResolvedStatus::Absorbed
        )
}

fn print_report(infos: &[WorktreeInfo]) {
    println!(
        "{:<20} {:<24} {:<18} {:<10} {:<14} {:<10} {:>10} {:>10}  CLASS",
        "REPO", "WORKTREE", "BRANCH", "STATUS", "PR", "DIRTY", "TOTAL", "ARTIFACTS"
    );
    let mut total_artifact_kb: u64 = 0;
    let mut total_safe_prune_kb: u64 = 0;
    for info in infos {
        total_artifact_kb += info.artifact_kb;
        if info.classification == Classification::SafePrune {
            total_safe_prune_kb += info.total_kb;
        }
        let mut basename = info
            .path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();
        if info.is_main {
            basename.push_str(" (main)");
        }
        println!(
            "{:<20} {:<24} {:<18} {:<10} {:<14} {:<10} {:>10} {:>10}  {}",
            truncate(&info.repo, 20),
            truncate(&basename, 24),
            truncate(&info.branch, 18),
            info.status.label(),
            truncate(&pr_column(info.pr), 14),
            truncate(&dirty_breakdown(&info.dirty_entries), 10),
            human_size(info.total_kb),
            human_size(info.artifact_kb),
            info.classification.label(),
        );
    }
    println!();
    println!(
        "total reclaimable artifacts: {}  |  total size in SAFE-PRUNE worktrees: {}",
        human_size(total_artifact_kb),
        human_size(total_safe_prune_kb)
    );

    let dirty_infos: Vec<&WorktreeInfo> = infos.iter().filter(|i| !i.dirty_entries.is_empty()).collect();
    if !dirty_infos.is_empty() {
        println!("\nREVIEW DETAILS");
        for info in dirty_infos {
            let flag = if would_be_safe_if_clean(info) {
                "  [MERGED BUT DIRTY]"
            } else {
                ""
            };
            println!(
                "\n{} ({}){}",
                info.path.display(),
                pr_column(info.pr),
                flag
            );
            const SHOWN: usize = 15;
            for entry in info.dirty_entries.iter().take(SHOWN) {
                println!("  {} {}", entry.code, entry.path);
            }
            let overflow = summarize_overflow(&info.dirty_entries, SHOWN);
            if !overflow.is_empty() {
                println!("  {overflow}");
            }
        }
    }
}

fn clean_artifacts(infos: &[WorktreeInfo], dry_run: bool) {
    let mut reclaimed_kb: u64 = 0;
    for info in infos {
        for rel in ARTIFACT_DIRS {
            let dir = info.path.join(rel);
            if !dir.is_dir() {
                continue;
            }
            let size = du_kb(&dir).unwrap_or(0);
            if dry_run {
                println!("[dry-run] would remove {} ({})", dir.display(), human_size(size));
                continue;
            }
            // Bazel and similar tools leave read-only trees; force writable first.
            let _ = run(None, &["chmod", "-R", "u+w", dir.to_str().unwrap_or_default()]);
            if fs::remove_dir_all(&dir).is_ok() {
                reclaimed_kb += size;
                println!("removed {} ({})", dir.display(), human_size(size));
            } else {
                eprintln!("worktree-triage: failed to remove {}", dir.display());
            }
        }
    }
    if !dry_run {
        println!("total reclaimed: {}", human_size(reclaimed_kb));
    }
}

fn prune_safe(infos: &[WorktreeInfo], dry_run: bool) {
    for info in infos {
        if info.classification != Classification::SafePrune {
            continue;
        }
        let repo_root = match find_repo_root(info) {
            Some(r) => r,
            None => continue,
        };
        if dry_run {
            println!("[dry-run] would remove worktree {}", info.path.display());
            continue;
        }
        let path_str = info.path.to_string_lossy().to_string();
        let ok = git_ok(&repo_root, &["worktree", "remove", &path_str]);
        if ok {
            println!("removed worktree {}", info.path.display());
        } else {
            eprintln!("worktree-triage: failed to remove worktree {}", info.path.display());
        }
    }
}

/// Best-effort recovery of the repo root a worktree belongs to, by asking
/// git itself rather than re-deriving it from the scan.
fn find_repo_root(info: &WorktreeInfo) -> Option<PathBuf> {
    let out = git_out(&info.path, &["rev-parse", "--git-common-dir"])?;
    let common_dir = PathBuf::from(out.trim());
    let common_dir = if common_dir.is_absolute() {
        common_dir
    } else {
        info.path.join(common_dir)
    };
    common_dir.parent().map(|p| p.to_path_buf())
}

fn main() {
    let argv: Vec<String> = env::args().skip(1).collect();
    let args = match parse_args(&argv) {
        Ok(a) => a,
        Err(e) => {
            eprintln!("{e}");
            exit(if e == HELP { 0 } else { 2 });
        }
    };

    if !args.root.is_dir() {
        eprintln!("worktree-triage: root not found: {}", args.root.display());
        exit(1);
    }

    let infos = gather(&args.root, !args.no_gh);

    if !args.clean_artifacts && !args.prune_safe {
        print_report(&infos);
        return;
    }

    if args.clean_artifacts {
        clean_artifacts(&infos, args.dry_run);
    }
    if args.prune_safe {
        prune_safe(&infos, args.dry_run);
    }
}

// ---------------------------------------------------------------------
// tests
// ---------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // -- classify_status --------------------------------------------------

    #[test]
    fn classify_status_ancestor_is_merged() {
        assert_eq!(classify_status(true, false), MergeStatus::Merged);
    }

    #[test]
    fn classify_status_ancestor_ignores_diff_flag() {
        assert_eq!(classify_status(true, true), MergeStatus::Merged);
    }

    #[test]
    fn classify_status_not_ancestor_but_empty_diff_is_absorbed() {
        assert_eq!(classify_status(false, true), MergeStatus::Absorbed);
    }

    #[test]
    fn classify_status_not_ancestor_and_nonempty_diff_is_unmerged() {
        assert_eq!(classify_status(false, false), MergeStatus::Unmerged);
    }

    // -- classify -----------------------------------------------------------

    #[test]
    fn classify_merged_clean_is_safe_prune() {
        assert_eq!(classify(ResolvedStatus::Merged, 0), Classification::SafePrune);
    }

    #[test]
    fn classify_absorbed_clean_is_safe_prune() {
        assert_eq!(classify(ResolvedStatus::Absorbed, 0), Classification::SafePrune);
    }

    #[test]
    fn classify_merged_dirty_is_review() {
        assert_eq!(classify(ResolvedStatus::Merged, 3), Classification::Review);
    }

    #[test]
    fn classify_absorbed_dirty_is_review() {
        assert_eq!(classify(ResolvedStatus::Absorbed, 1), Classification::Review);
    }

    #[test]
    fn classify_unmerged_clean_is_review() {
        assert_eq!(classify(ResolvedStatus::Unmerged, 0), Classification::Review);
    }

    #[test]
    fn classify_unmerged_dirty_is_review() {
        assert_eq!(classify(ResolvedStatus::Unmerged, 5), Classification::Review);
    }

    #[test]
    fn classify_pr_merged_clean_non_primary_is_safe_prune() {
        assert_eq!(classify(ResolvedStatus::PrMerged, 0), Classification::SafePrune);
    }

    #[test]
    fn classify_pr_merged_dirty_is_review() {
        assert_eq!(classify(ResolvedStatus::PrMerged, 2), Classification::Review);
    }

    #[test]
    fn classify_pr_open_is_always_review() {
        assert_eq!(classify(ResolvedStatus::PrOpen, 0), Classification::Review);
    }

    #[test]
    fn classify_pr_closed_is_always_review() {
        assert_eq!(classify(ResolvedStatus::PrClosed, 0), Classification::Review);
    }

    // -- pr_state_to_status / resolve_status ---------------------------------

    #[test]
    fn pr_state_to_status_maps_each_variant() {
        assert_eq!(pr_state_to_status(Some(PrState::Merged)), Some(ResolvedStatus::PrMerged));
        assert_eq!(pr_state_to_status(Some(PrState::Open)), Some(ResolvedStatus::PrOpen));
        assert_eq!(pr_state_to_status(Some(PrState::Closed)), Some(ResolvedStatus::PrClosed));
    }

    #[test]
    fn pr_state_to_status_none_falls_back_to_local() {
        assert_eq!(pr_state_to_status(None), None);
    }

    #[test]
    fn resolve_status_pr_open_wins_over_local_ancestor() {
        // Open PR is authoritative even though local git looks merged.
        assert_eq!(resolve_status(Some(PrState::Open), true, true), ResolvedStatus::PrOpen);
    }

    #[test]
    fn resolve_status_no_pr_falls_back_to_local_unmerged() {
        assert_eq!(resolve_status(None, false, false), ResolvedStatus::Unmerged);
    }

    #[test]
    fn resolve_status_no_pr_falls_back_to_local_merged() {
        assert_eq!(resolve_status(None, true, false), ResolvedStatus::Merged);
    }

    // -- parse_origin_head ---------------------------------------------------

    #[test]
    fn parse_origin_head_extracts_branch() {
        assert_eq!(
            parse_origin_head("refs/remotes/origin/main"),
            Some("main".to_string())
        );
    }

    #[test]
    fn parse_origin_head_handles_master() {
        assert_eq!(
            parse_origin_head("refs/remotes/origin/master\n"),
            Some("master".to_string())
        );
    }

    #[test]
    fn parse_origin_head_rejects_non_origin_ref() {
        assert_eq!(parse_origin_head("refs/heads/main"), None);
    }

    #[test]
    fn parse_origin_head_rejects_empty() {
        assert_eq!(parse_origin_head(""), None);
        assert_eq!(parse_origin_head("refs/remotes/origin/"), None);
    }

    // -- human_size -----------------------------------------------------------

    #[test]
    fn human_size_bytes_range() {
        assert_eq!(human_size(0), "0KB");
        assert_eq!(human_size(512), "512KB");
        assert_eq!(human_size(1023), "1023KB");
    }

    #[test]
    fn human_size_megabytes_range() {
        assert_eq!(human_size(1024), "1.0MB");
        assert_eq!(human_size(2048), "2.0MB");
        assert_eq!(human_size(1024 * 500), "500.0MB");
    }

    #[test]
    fn human_size_gigabytes_range() {
        assert_eq!(human_size(1024 * 1024), "1.00GB");
        assert_eq!(human_size(1024 * 1024 * 3), "3.00GB");
    }

    // -- parse_worktree_porcelain ----------------------------------------------

    const SAMPLE_PORCELAIN: &str = "worktree /Users/rch/repo/occult\nHEAD abc123\nbranch refs/heads/main\n\nworktree /Users/rch/repo/occult-wt-129\nHEAD def456\nbranch refs/heads/rch/feature/thing\n\nworktree /Users/rch/repo/occult-wt-detached\nHEAD 789abc\ndetached\n\n";

    #[test]
    fn parse_porcelain_extracts_all_records() {
        let records = parse_worktree_porcelain(SAMPLE_PORCELAIN);
        assert_eq!(records.len(), 3);
    }

    #[test]
    fn parse_porcelain_main_worktree_fields() {
        let records = parse_worktree_porcelain(SAMPLE_PORCELAIN);
        assert_eq!(records[0].path, PathBuf::from("/Users/rch/repo/occult"));
        assert_eq!(records[0].head, "abc123");
        assert_eq!(records[0].branch, Some("main".to_string()));
    }

    #[test]
    fn parse_porcelain_strips_refs_heads_prefix() {
        let records = parse_worktree_porcelain(SAMPLE_PORCELAIN);
        assert_eq!(records[1].branch, Some("rch/feature/thing".to_string()));
    }

    #[test]
    fn parse_porcelain_detached_has_no_branch() {
        let records = parse_worktree_porcelain(SAMPLE_PORCELAIN);
        assert_eq!(records[2].branch, None);
        assert_eq!(records[2].head, "789abc");
    }

    #[test]
    fn parse_porcelain_skips_bare_repo() {
        let input = "worktree /Users/rch/repo/bare.git\nbare\n\nworktree /Users/rch/repo/bare-wt\nHEAD abc\nbranch refs/heads/main\n\n";
        let records = parse_worktree_porcelain(input);
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].path, PathBuf::from("/Users/rch/repo/bare-wt"));
    }

    #[test]
    fn parse_porcelain_handles_no_trailing_blank_line() {
        let input = "worktree /a\nHEAD abc\nbranch refs/heads/main";
        let records = parse_worktree_porcelain(input);
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].path, PathBuf::from("/a"));
    }

    #[test]
    fn parse_porcelain_empty_input() {
        assert!(parse_worktree_porcelain("").is_empty());
    }

    // -- parse_status_porcelain / dirty_breakdown ----------------------------

    #[test]
    fn parse_status_porcelain_empty_is_empty() {
        assert!(parse_status_porcelain("").is_empty());
    }

    const SAMPLE_STATUS: &str =
        " M modified.rs\n?? untracked.rs\nA  staged-new.rs\n D deleted.rs\nR  old.rs -> new.rs\n";

    #[test]
    fn parse_status_porcelain_extracts_all_entries() {
        let entries = parse_status_porcelain(SAMPLE_STATUS);
        assert_eq!(entries.len(), 5);
    }

    #[test]
    fn parse_status_porcelain_splits_code_and_path() {
        let entries = parse_status_porcelain(SAMPLE_STATUS);
        assert_eq!(entries[0].code, " M");
        assert_eq!(entries[0].path, "modified.rs");
        assert_eq!(entries[1].code, "??");
        assert_eq!(entries[1].path, "untracked.rs");
    }

    #[test]
    fn parse_status_porcelain_keeps_rename_arrow_in_path() {
        let entries = parse_status_porcelain(SAMPLE_STATUS);
        assert_eq!(entries[4].code, "R ");
        assert_eq!(entries[4].path, "old.rs -> new.rs");
    }

    #[test]
    fn dirty_breakdown_empty_is_clean() {
        assert_eq!(dirty_breakdown(&[]), "clean");
    }

    #[test]
    fn dirty_breakdown_summarizes_by_kind() {
        let entries = parse_status_porcelain(" M a.rs\n M b.rs\n?? c.rs\n");
        assert_eq!(dirty_breakdown(&entries), "2M 1??");
    }

    #[test]
    fn dirty_breakdown_includes_deleted_and_added() {
        let entries = parse_status_porcelain("A  a.rs\n D b.rs\n");
        assert_eq!(dirty_breakdown(&entries), "1A 1D");
    }

    // -- summarize_overflow -----------------------------------------------

    fn entries_of(codes: &[&str]) -> Vec<StatusEntry> {
        codes
            .iter()
            .enumerate()
            .map(|(i, code)| StatusEntry {
                code: code.to_string(),
                path: format!("file{i}.rs"),
            })
            .collect()
    }

    #[test]
    fn summarize_overflow_empty_when_at_or_under_cap() {
        let entries = entries_of(&[" M", " M", "??"]);
        assert_eq!(summarize_overflow(&entries, 15), "");
        let exactly_15: Vec<StatusEntry> = (0..15)
            .map(|i| StatusEntry {
                code: " M".to_string(),
                path: format!("f{i}.rs"),
            })
            .collect();
        assert_eq!(summarize_overflow(&exactly_15, 15), "");
    }

    #[test]
    fn summarize_overflow_breaks_down_remaining_by_category() {
        // 15 shown, then 1180 deletions, 1 modification, 1 untracked beyond.
        let mut codes = vec![" M"; 15];
        codes.extend(std::iter::repeat_n(" D", 1180));
        codes.push(" M");
        codes.push("??");
        let entries = entries_of(&codes);
        assert_eq!(
            summarize_overflow(&entries, 15),
            "... and 1,182 more (M:1 D:1180 ??:1)"
        );
    }

    #[test]
    fn summarize_overflow_omits_zero_categories() {
        let codes: Vec<&str> = std::iter::repeat_n("??", 20).collect();
        let entries = entries_of(&codes);
        assert_eq!(summarize_overflow(&entries, 15), "... and 5 more (??:5)");
    }

    // -- gh JSON parsing ------------------------------------------------------

    const SAMPLE_GH_JSON: &str = r#"[
        {"number":123,"headRefName":"rch/feature/thing","state":"MERGED","mergedAt":"2026-01-01T00:00:00Z"},
        {"number":124,"headRefName":"rch/wip/other","state":"OPEN","mergedAt":null},
        {"number":125,"headRefName":"rch/abandoned","state":"CLOSED","mergedAt":null}
    ]"#;

    #[test]
    fn parse_pr_list_json_extracts_all_records() {
        let records = parse_pr_list_json(SAMPLE_GH_JSON);
        assert_eq!(records.len(), 3);
    }

    #[test]
    fn parse_pr_list_json_maps_states() {
        let records = parse_pr_list_json(SAMPLE_GH_JSON);
        assert_eq!(records[0].state, PrState::Merged);
        assert_eq!(records[1].state, PrState::Open);
        assert_eq!(records[2].state, PrState::Closed);
    }

    #[test]
    fn parse_pr_list_json_extracts_branch_and_number() {
        let records = parse_pr_list_json(SAMPLE_GH_JSON);
        assert_eq!(records[0].head_ref_name, "rch/feature/thing");
        assert_eq!(records[0].number, 123);
    }

    #[test]
    fn parse_pr_list_json_empty_array() {
        assert!(parse_pr_list_json("[]").is_empty());
    }

    // -- repo_slug_from_origin_url --------------------------------------------

    #[test]
    fn repo_slug_from_ssh_url() {
        assert_eq!(
            repo_slug_from_origin_url("git@github.com:owner/repo.git"),
            Some("owner/repo".to_string())
        );
    }

    #[test]
    fn repo_slug_from_https_url() {
        assert_eq!(
            repo_slug_from_origin_url("https://github.com/owner/repo.git"),
            Some("owner/repo".to_string())
        );
    }

    #[test]
    fn repo_slug_from_https_url_without_git_suffix() {
        assert_eq!(
            repo_slug_from_origin_url("https://github.com/owner/repo"),
            Some("owner/repo".to_string())
        );
    }

    #[test]
    fn repo_slug_from_unrecognized_scheme_is_none() {
        assert_eq!(repo_slug_from_origin_url("file:///local/path"), None);
    }

    // -- artifact_kb_for ------------------------------------------------------

    #[test]
    fn artifact_kb_for_sums_existing_dirs() {
        let dir = std::env::temp_dir().join(format!("wt-triage-test-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(dir.join("target")).unwrap();
        fs::create_dir_all(dir.join("dist")).unwrap();
        let total = artifact_kb_for(&dir, |p| {
            if p.ends_with("target") {
                Some(100)
            } else if p.ends_with("dist") {
                Some(50)
            } else {
                None
            }
        });
        assert_eq!(total, 150);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn artifact_kb_for_ignores_missing_dirs() {
        let dir = std::env::temp_dir().join(format!("wt-triage-test-empty-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let total = artifact_kb_for(&dir, |_| Some(999));
        assert_eq!(total, 0);
        let _ = fs::remove_dir_all(&dir);
    }

    // -- truncate -------------------------------------------------------------

    #[test]
    fn truncate_leaves_short_strings_alone() {
        assert_eq!(truncate("short", 20), "short");
    }

    #[test]
    fn truncate_shortens_long_strings_with_ellipsis() {
        assert_eq!(truncate("rch/feature/a-very-long-branch-name", 20), "rch/feature/a-ver...");
    }

    // -- parse_args -----------------------------------------------------------

    #[test]
    fn parse_args_defaults_to_report_mode() {
        let args = parse_args(&[]).unwrap();
        assert!(!args.clean_artifacts);
        assert!(!args.prune_safe);
        assert!(!args.dry_run);
        assert_eq!(args.root, PathBuf::from(DEFAULT_ROOT));
    }

    #[test]
    fn parse_args_accepts_action_flags() {
        let argv: Vec<String> = ["--clean-artifacts", "--dry-run"]
            .iter()
            .map(|s| s.to_string())
            .collect();
        let args = parse_args(&argv).unwrap();
        assert!(args.clean_artifacts);
        assert!(args.dry_run);
        assert!(!args.prune_safe);
    }

    #[test]
    fn parse_args_accepts_custom_root() {
        let argv: Vec<String> = ["--root", "/tmp/somewhere"]
            .iter()
            .map(|s| s.to_string())
            .collect();
        let args = parse_args(&argv).unwrap();
        assert_eq!(args.root, PathBuf::from("/tmp/somewhere"));
    }

    #[test]
    fn parse_args_rejects_unknown_flag() {
        let argv: Vec<String> = ["--bogus".to_string()].to_vec();
        assert!(parse_args(&argv).is_err());
    }

    #[test]
    fn parse_args_root_without_value_errors() {
        let argv: Vec<String> = ["--root".to_string()].to_vec();
        assert!(parse_args(&argv).is_err());
    }
}
