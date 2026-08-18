//! worktree-triage — report on and safely reclaim disk space from git
//! worktrees living under a repo root (default `~/repo`).
//! Std-only: shells out to `git` and `du`; no external crates.
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
//!
//! Discovery: each immediate subdirectory of the root whose `.git` is a
//! directory (a real checkout, not a linked worktree) is treated as a repo.
//! Its worktrees, including ones registered as sibling directories and not
//! only ones nested under `.worktrees/`, are enumerated via
//! `git worktree list --porcelain`, which reports absolute paths regardless
//! of where they physically live.
//!
//! Safety classification (conservative: a false "safe" causes data loss):
//!   MERGED    if the worktree's HEAD is an ancestor of the repo's main branch.
//!   ABSORBED  if not MERGED, but `main...HEAD` has an empty diff (covers
//!             squash-merges: the branch introduces nothing main lacks).
//!   UNMERGED  otherwise.
//!   SAFE-PRUNE = (MERGED or ABSORBED) and the worktree has no uncommitted
//!                changes. Everything else is REVIEW. The repo's own main
//!                worktree is always REVIEW (never a `--prune-safe` target)
//!                regardless of merge status.
//!
//! Build artifacts (target/, build/, node_modules/, dist/, src-tauri/target/)
//! are regenerable and reported/cleanable on every worktree, safe or not.

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

impl MergeStatus {
    fn label(self) -> &'static str {
        match self {
            MergeStatus::Merged => "MERGED",
            MergeStatus::Absorbed => "ABSORBED",
            MergeStatus::Unmerged => "UNMERGED",
        }
    }
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
    status: MergeStatus,
    dirty: usize,
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

/// SAFE-PRUNE iff (MERGED or ABSORBED) and no uncommitted changes.
fn classify(status: MergeStatus, dirty: usize) -> Classification {
    match status {
        MergeStatus::Unmerged => Classification::Review,
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

/// Count non-empty lines of `git status --porcelain` output.
fn count_dirty_lines(output: &str) -> usize {
    output.lines().filter(|l| !l.is_empty()).count()
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

fn build_worktree_info(repo: &Path, repo_name: &str, main_branch: &str, rec: &WorktreeRecord) -> Option<WorktreeInfo> {
    if !rec.path.is_dir() {
        return None;
    }
    let dirty = git_out(&rec.path, &["status", "--porcelain"])
        .map(|out| count_dirty_lines(&out))
        .unwrap_or(0);

    let is_ancestor = git_ok(repo, &["merge-base", "--is-ancestor", &rec.head, main_branch]);
    let diff_empty = is_ancestor
        || git_ok(repo, &["diff", "--quiet", &format!("{main_branch}...{}", rec.head)]);
    let status = classify_status(is_ancestor, diff_empty);

    let is_main = same_path(&rec.path, repo);
    let mut classification = classify(status, dirty);
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
        dirty,
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
}

fn parse_args(argv: &[String]) -> Result<Args, String> {
    let mut root = PathBuf::from(DEFAULT_ROOT);
    let mut clean_artifacts = false;
    let mut prune_safe = false;
    let mut dry_run = false;

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
    })
}

const HELP: &str = "\
worktree-triage [--root PATH] [--clean-artifacts] [--prune-safe] [--dry-run]

  (no flags)          read-only report, never mutates anything
  --clean-artifacts   delete target/build/node_modules/dist from every worktree
  --prune-safe        git worktree remove on SAFE-PRUNE worktrees only
  --dry-run           combined with an action flag, print without doing it
  --root PATH         scan a different repo root (default /Users/rch/repo)";

// ---------------------------------------------------------------------
// main
// ---------------------------------------------------------------------

fn gather(root: &Path) -> Vec<WorktreeInfo> {
    let mut infos = Vec::new();
    for repo in discover_repos(root) {
        let repo_name = repo
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();
        let Some(main_branch) = resolve_main_branch(&repo) else {
            continue;
        };
        for rec in list_worktrees(&repo) {
            if let Some(info) = build_worktree_info(&repo, &repo_name, &main_branch, &rec) {
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

fn print_report(infos: &[WorktreeInfo]) {
    println!(
        "{:<20} {:<24} {:<18} {:<9} {:<5} {:>10} {:>10}  CLASS",
        "REPO", "WORKTREE", "BRANCH", "STATUS", "DIRTY", "TOTAL", "ARTIFACTS"
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
            "{:<20} {:<24} {:<18} {:<9} {:<5} {:>10} {:>10}  {}",
            truncate(&info.repo, 20),
            truncate(&basename, 24),
            truncate(&info.branch, 18),
            info.status.label(),
            if info.dirty > 0 { "Y" } else { "N" },
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

    let infos = gather(&args.root);

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
        assert_eq!(classify(MergeStatus::Merged, 0), Classification::SafePrune);
    }

    #[test]
    fn classify_absorbed_clean_is_safe_prune() {
        assert_eq!(classify(MergeStatus::Absorbed, 0), Classification::SafePrune);
    }

    #[test]
    fn classify_merged_dirty_is_review() {
        assert_eq!(classify(MergeStatus::Merged, 3), Classification::Review);
    }

    #[test]
    fn classify_absorbed_dirty_is_review() {
        assert_eq!(classify(MergeStatus::Absorbed, 1), Classification::Review);
    }

    #[test]
    fn classify_unmerged_clean_is_review() {
        assert_eq!(classify(MergeStatus::Unmerged, 0), Classification::Review);
    }

    #[test]
    fn classify_unmerged_dirty_is_review() {
        assert_eq!(classify(MergeStatus::Unmerged, 5), Classification::Review);
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

    // -- count_dirty_lines --------------------------------------------------

    #[test]
    fn count_dirty_lines_empty_is_zero() {
        assert_eq!(count_dirty_lines(""), 0);
    }

    #[test]
    fn count_dirty_lines_counts_nonempty_lines() {
        assert_eq!(count_dirty_lines(" M foo.rs\n?? bar.rs\n"), 2);
    }

    #[test]
    fn count_dirty_lines_ignores_trailing_blank() {
        assert_eq!(count_dirty_lines(" M foo.rs\n\n"), 1);
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
