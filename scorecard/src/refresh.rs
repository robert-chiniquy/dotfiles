use super::{chart, is_banner_label, is_header, is_sep, parse_links, split_row, split_weight, Seg};
use std::collections::{HashMap, HashSet};
use std::env;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const DEFAULT_REFRESH_INTERVAL_SECS: u64 = 60 * 60;
const PROVIDER_BATCH_SIZE: usize = 50;

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
enum TrackerRef {
    GitHub {
        owner: String,
        repo: String,
        number: u64,
    },
    Linear {
        identifier: String,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum TrackerState {
    Open,
    Terminal,
    Unknown,
}

#[derive(Clone, Debug)]
struct TrackedRow {
    id: String,
    refs: Vec<TrackerRef>,
}

#[derive(Default)]
pub(crate) struct RefreshReport {
    linked_rows: usize,
    total_refs: usize,
    checked_refs: usize,
    removed_rows: usize,
    warnings: Vec<String>,
}

fn tracker_ref(url: &str) -> Option<TrackerRef> {
    let url = url.split(['?', '#']).next().unwrap_or(url);
    let rest = url
        .strip_prefix("https://")
        .or_else(|| url.strip_prefix("http://"))?;
    let (host, path) = rest.split_once('/')?;
    let parts: Vec<&str> = path.split('/').filter(|part| !part.is_empty()).collect();

    if host.eq_ignore_ascii_case("github.com") {
        if parts.len() >= 4 && parts[2] == "pull" {
            return Some(TrackerRef::GitHub {
                owner: parts[0].to_string(),
                repo: parts[1].to_string(),
                number: parts[3].parse().ok()?,
            });
        }
        return None;
    }

    if host.eq_ignore_ascii_case("linear.app") {
        let issue = parts.iter().position(|part| *part == "issue")?;
        let identifier = parts.get(issue + 1)?.trim();
        if !identifier.is_empty() {
            return Some(TrackerRef::Linear {
                identifier: identifier.to_string(),
            });
        }
    }
    None
}

fn tracked_row(cells: &[String]) -> Option<TrackedRow> {
    if cells.len() < 4 || is_sep(cells) || is_header(cells) {
        return None;
    }
    let mut seen = HashSet::new();
    let mut refs = Vec::new();
    for cell in cells {
        for seg in parse_links(cell) {
            if let Seg::Link { url, .. } = seg {
                if let Some(reference) = tracker_ref(&url) {
                    if seen.insert(reference.clone()) {
                        refs.push(reference);
                    }
                }
            }
        }
    }
    (!refs.is_empty()).then(|| TrackedRow {
        id: cells[0].clone(),
        refs,
    })
}

fn tracked_rows(input: &str) -> Vec<TrackedRow> {
    let mut rows = Vec::new();
    let mut in_criteria = false;
    for raw in input.lines() {
        let trimmed = raw.trim();
        if let Some(label) = trimmed.strip_prefix("## ") {
            in_criteria =
                !is_banner_label(&split_weight(label).0) && !chart::is_chart_heading(label);
            continue;
        }
        if in_criteria && trimmed.starts_with('|') {
            if let Some(row) = tracked_row(&split_row(trimmed)) {
                rows.push(row);
            }
        }
    }
    rows
}

pub(crate) fn has_tracker_rows(input: &str) -> bool {
    !tracked_rows(input).is_empty()
}

fn github_state(state: &str) -> TrackerState {
    match state.trim().to_ascii_uppercase().as_str() {
        "CLOSED" | "MERGED" => TrackerState::Terminal,
        "OPEN" => TrackerState::Open,
        _ => TrackerState::Unknown,
    }
}

fn linear_state(state: &str) -> TrackerState {
    match state.trim().to_ascii_lowercase().as_str() {
        "completed" | "canceled" => TrackerState::Terminal,
        "triage" | "backlog" | "unstarted" | "started" => TrackerState::Open,
        _ => TrackerState::Unknown,
    }
}

fn row_is_terminal(row: &TrackedRow, states: &HashMap<TrackerRef, TrackerState>) -> bool {
    !row.id.trim().is_empty()
        && !row.refs.is_empty()
        && row
            .refs
            .iter()
            .all(|reference| states.get(reference) == Some(&TrackerState::Terminal))
}

fn prune_terminal_content(
    source: &str,
    states: &HashMap<TrackerRef, TrackerState>,
) -> (String, usize) {
    let mut kept = Vec::new();
    let mut removed = 0;
    let mut in_criteria = false;
    for raw in source.lines() {
        let trimmed = raw.trim();
        if let Some(label) = trimmed.strip_prefix("## ") {
            in_criteria =
                !is_banner_label(&split_weight(label).0) && !chart::is_chart_heading(label);
        }
        let row = (in_criteria && trimmed.starts_with('|'))
            .then(|| split_row(trimmed))
            .and_then(|cells| tracked_row(&cells));
        if row
            .as_ref()
            .map(|row| row_is_terminal(row, states))
            .unwrap_or(false)
        {
            removed += 1;
        } else {
            kept.push(raw);
        }
    }

    if removed == 0 {
        return (source.to_string(), 0);
    }
    let mut updated = kept.join("\n");
    if source.ends_with('\n') {
        updated.push('\n');
    }
    (updated, removed)
}

fn write_atomic(path: &Path, expected: &str, content: &str) -> Result<(), String> {
    let parent = path
        .parent()
        .ok_or_else(|| format!("{} has no parent directory", path.display()))?;
    let name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("status.md");
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let temporary = parent.join(format!(
        ".{}.scorecard-{}-{}.tmp",
        name,
        std::process::id(),
        nonce
    ));

    let result = (|| -> Result<(), String> {
        let mut output = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temporary)
            .map_err(|e| format!("create {}: {}", temporary.display(), e))?;
        output
            .write_all(content.as_bytes())
            .map_err(|e| format!("write {}: {}", temporary.display(), e))?;
        if let Ok(metadata) = std::fs::metadata(path) {
            let _ = std::fs::set_permissions(&temporary, metadata.permissions());
        }
        output
            .sync_all()
            .map_err(|e| format!("sync {}: {}", temporary.display(), e))?;
        let live = std::fs::read_to_string(path)
            .map_err(|error| format!("re-read {}: {}", path.display(), error))?;
        if live != expected {
            return Err(format!(
                "{} changed while tracker states were being applied; kept the newer version",
                path.display()
            ));
        }
        std::fs::rename(&temporary, path).map_err(|e| format!("replace {}: {}", path.display(), e))
    })();
    if result.is_err() {
        let _ = std::fs::remove_file(&temporary);
    }
    result
}

fn prune_terminal_rows(
    file: &str,
    states: &HashMap<TrackerRef, TrackerState>,
) -> Result<usize, String> {
    let current =
        std::fs::read_to_string(file).map_err(|error| format!("read {}: {}", file, error))?;
    let (updated, removed) = prune_terminal_content(&current, states);
    if removed > 0 {
        write_atomic(Path::new(file), &current, &updated)?;
    }
    Ok(removed)
}

fn json_escape(value: &str) -> String {
    let mut escaped = String::new();
    for ch in value.chars() {
        match ch {
            '"' => escaped.push_str("\\\""),
            '\\' => escaped.push_str("\\\\"),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            ch if ch <= '\u{1f}' => escaped.push_str(&format!("\\u{:04x}", ch as u32)),
            ch => escaped.push(ch),
        }
    }
    escaped
}

fn curl_config_quote(value: &str) -> String {
    let mut escaped = String::new();
    for ch in value.chars() {
        match ch {
            '"' => escaped.push_str("\\\""),
            '\\' => escaped.push_str("\\\\"),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            ch => escaped.push(ch),
        }
    }
    escaped
}

fn brief_error(bytes: &[u8]) -> String {
    let compact = String::from_utf8_lossy(bytes)
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    if compact.chars().count() <= 180 {
        compact
    } else {
        format!("{}…", compact.chars().take(179).collect::<String>())
    }
}

fn check_github(
    refs: &[TrackerRef],
    states: &mut HashMap<TrackerRef, TrackerState>,
) -> Result<(), String> {
    let github: Vec<TrackerRef> = refs
        .iter()
        .filter(|reference| matches!(reference, TrackerRef::GitHub { .. }))
        .cloned()
        .collect();
    if github.is_empty() {
        return Ok(());
    }

    for chunk in github.chunks(PROVIDER_BATCH_SIZE) {
        let mut query = String::from("query {");
        for (index, reference) in chunk.iter().enumerate() {
            if let TrackerRef::GitHub {
                owner,
                repo,
                number,
            } = reference
            {
                query.push_str(&format!(
                    " g{}: repository(owner:\"{}\", name:\"{}\") {{ pullRequest(number:{}) {{ state }} }}",
                    index,
                    json_escape(owner),
                    json_escape(repo),
                    number
                ));
            }
        }
        query.push_str(" }");

        let output = Command::new("gh")
            .args(["api", "graphql", "-f"])
            .arg(format!("query={}", query))
            .args([
                "--jq",
                ".data | to_entries[] | [.key, (.value.pullRequest.state // \"UNKNOWN\")] | @tsv",
            ])
            .env("GH_PROMPT_DISABLED", "1")
            .env("GH_PAGER", "cat")
            .env("GH_NO_UPDATE_NOTIFIER", "1")
            .stdin(Stdio::null())
            .output()
            .map_err(|error| format!("cannot run gh: {}", error))?;
        if !output.status.success() {
            return Err(format!(
                "gh GraphQL failed: {}",
                brief_error(&output.stderr)
            ));
        }

        let mut returned = 0;
        for line in String::from_utf8_lossy(&output.stdout).lines() {
            let Some((alias, state)) = line.split_once('\t') else {
                continue;
            };
            let Some(index) = alias
                .strip_prefix('g')
                .and_then(|number| number.parse::<usize>().ok())
            else {
                continue;
            };
            if let Some(reference) = chunk.get(index) {
                states.insert(reference.clone(), github_state(state));
                returned += 1;
            }
        }
        if returned == 0 {
            return Err("gh GraphQL returned no pull-request states".into());
        }
    }
    Ok(())
}

fn jq_linear_states(json: &[u8]) -> Result<String, String> {
    let mut child = Command::new("jq")
        .args([
            "-r",
            ".data | to_entries[] | [.key, (.value.state.type // \"unknown\")] | @tsv",
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| format!("cannot run jq: {}", error))?;
    child
        .stdin
        .take()
        .ok_or_else(|| "cannot open jq stdin".to_string())?
        .write_all(json)
        .map_err(|error| format!("write jq input: {}", error))?;
    let output = child
        .wait_with_output()
        .map_err(|error| format!("wait for jq: {}", error))?;
    if !output.status.success() {
        return Err(format!(
            "Linear response parse failed: {}",
            brief_error(&output.stderr)
        ));
    }
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

fn linear_authorization() -> Result<String, String> {
    env::var("LINEAR_API_KEY")
        .ok()
        .filter(|key| !key.trim().is_empty())
        .or_else(|| {
            env::var("LINEAR_ACCESS_TOKEN")
                .ok()
                .filter(|token| !token.trim().is_empty())
                .map(|token| format!("Bearer {}", token))
        })
        .ok_or_else(|| {
            "set LINEAR_API_KEY (or LINEAR_ACCESS_TOKEN) to check Linear links".to_string()
        })
}

fn linear_curl_config(authorization: &str, query: &str) -> String {
    let body = format!("{{\"query\":\"{}\"}}", json_escape(query));
    format!(
        "silent\nshow-error\nfail-with-body\nconnect-timeout = 3\nmax-time = 10\nrequest = \"POST\"\nurl = \"https://api.linear.app/graphql\"\nheader = \"Content-Type: application/json\"\nheader = \"Authorization: {}\"\ndata = \"{}\"\n",
        curl_config_quote(authorization),
        curl_config_quote(&body)
    )
}

fn check_linear(
    refs: &[TrackerRef],
    states: &mut HashMap<TrackerRef, TrackerState>,
) -> Result<(), String> {
    let linear: Vec<TrackerRef> = refs
        .iter()
        .filter(|reference| matches!(reference, TrackerRef::Linear { .. }))
        .cloned()
        .collect();
    if linear.is_empty() {
        return Ok(());
    }
    let authorization = linear_authorization()?;

    for chunk in linear.chunks(PROVIDER_BATCH_SIZE) {
        let mut query = String::from("query {");
        for (index, reference) in chunk.iter().enumerate() {
            if let TrackerRef::Linear { identifier } = reference {
                query.push_str(&format!(
                    " l{}: issue(id:\"{}\") {{ state {{ type }} }}",
                    index,
                    json_escape(identifier)
                ));
            }
        }
        query.push_str(" }");
        let config = linear_curl_config(&authorization, &query);

        let mut child = Command::new("curl")
            .args(["--config", "-"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|error| format!("cannot run curl: {}", error))?;
        child
            .stdin
            .take()
            .ok_or_else(|| "cannot open curl stdin".to_string())?
            .write_all(config.as_bytes())
            .map_err(|error| format!("write curl config: {}", error))?;
        let output = child
            .wait_with_output()
            .map_err(|error| format!("wait for curl: {}", error))?;
        if !output.status.success() {
            return Err(format!(
                "Linear GraphQL failed: {}",
                brief_error(&output.stderr)
            ));
        }

        let parsed = jq_linear_states(&output.stdout)?;
        let mut returned = 0;
        for line in parsed.lines() {
            let Some((alias, state)) = line.split_once('\t') else {
                continue;
            };
            let Some(index) = alias
                .strip_prefix('l')
                .and_then(|number| number.parse::<usize>().ok())
            else {
                continue;
            };
            if let Some(reference) = chunk.get(index) {
                states.insert(reference.clone(), linear_state(state));
                returned += 1;
            }
        }
        if returned == 0 {
            return Err("Linear GraphQL returned no issue states".into());
        }
    }
    Ok(())
}

fn canonical_file(file: &str) -> String {
    std::fs::canonicalize(file)
        .map(|path| path.to_string_lossy().into_owned())
        .unwrap_or_else(|_| file.to_string())
}

fn refresh_file(file: &str) -> Result<RefreshReport, String> {
    if !file.ends_with(".md") {
        return Err(format!("refusing to edit non-markdown file: {}", file));
    }
    let snapshot =
        std::fs::read_to_string(file).map_err(|error| format!("read {}: {}", file, error))?;
    let rows = tracked_rows(&snapshot);
    let mut report = RefreshReport {
        linked_rows: rows.len(),
        ..RefreshReport::default()
    };
    if rows.is_empty() {
        return Ok(report);
    }

    let mut unique = HashSet::new();
    for row in &rows {
        unique.extend(row.refs.iter().cloned());
    }
    let mut refs: Vec<TrackerRef> = unique.into_iter().collect();
    refs.sort();
    report.total_refs = refs.len();

    let mut states = HashMap::new();
    if let Err(error) = check_github(&refs, &mut states) {
        report.warnings.push(format!("GitHub: {}", error));
    }
    if let Err(error) = check_linear(&refs, &mut states) {
        report.warnings.push(format!("Linear: {}", error));
    }
    report.checked_refs = states
        .values()
        .filter(|state| **state != TrackerState::Unknown)
        .count();
    report.removed_rows = prune_terminal_rows(file, &states)?;
    Ok(report)
}

pub(crate) fn refresh(file: &str) -> Result<RefreshReport, String> {
    let file = canonical_file(file);
    let report = refresh_file(&file)?;
    mark_refreshed(&file);
    Ok(report)
}

pub(crate) fn summary(report: &RefreshReport) -> String {
    let mut summary = format!(
        "scorecard: checked {}/{} tracker links across {} row{}; removed {} row{}",
        report.checked_refs,
        report.total_refs,
        report.linked_rows,
        if report.linked_rows == 1 { "" } else { "s" },
        report.removed_rows,
        if report.removed_rows == 1 { "" } else { "s" }
    );
    for warning in &report.warnings {
        summary.push_str(&format!("\n  warning: {}", warning));
    }
    summary
}

pub(crate) fn default_file() -> String {
    env::var("SCORECARD_FILE").unwrap_or_else(|_| {
        format!(
            "{}/.config/scorecard/status.md",
            env::var("HOME").unwrap_or_default()
        )
    })
}

fn refresh_interval() -> Option<Duration> {
    if env::var("SCORECARD_AUTO_REFRESH")
        .ok()
        .map(|value| {
            matches!(
                value.trim().to_ascii_lowercase().as_str(),
                "0" | "false" | "off"
            )
        })
        .unwrap_or(false)
    {
        return None;
    }
    let seconds = env::var("SCORECARD_REFRESH_INTERVAL")
        .ok()
        .and_then(|value| value.trim().parse::<u64>().ok())
        .unwrap_or(DEFAULT_REFRESH_INTERVAL_SECS);
    (seconds > 0).then(|| Duration::from_secs(seconds))
}

fn path_hash(path: &str) -> u64 {
    let mut hash = 0xcbf29ce484222325u64;
    for byte in path.as_bytes() {
        hash ^= *byte as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

fn refresh_marker(file: &str) -> Option<PathBuf> {
    let root = env::var("XDG_CACHE_HOME")
        .ok()
        .filter(|path| !path.trim().is_empty())
        .map(PathBuf::from)
        .or_else(|| {
            env::var("HOME")
                .ok()
                .filter(|path| !path.trim().is_empty())
                .map(|home| PathBuf::from(home).join(".cache"))
        })?;
    Some(
        root.join("scorecard")
            .join(format!("refresh-{:016x}", path_hash(file))),
    )
}

fn source_stamp(file: &str) -> String {
    let Ok(metadata) = std::fs::metadata(file) else {
        return "missing".into();
    };
    let modified = metadata
        .modified()
        .ok()
        .and_then(|time| time.duration_since(UNIX_EPOCH).ok())
        .map(|duration| duration.as_nanos())
        .unwrap_or_default();
    format!("{}:{}", metadata.len(), modified)
}

fn mark_refreshed(file: &str) {
    let Some(marker) = refresh_marker(file) else {
        return;
    };
    if let Some(parent) = marker.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let _ = std::fs::write(marker, source_stamp(file));
}

pub(crate) fn schedule(file: &str) {
    let Some(interval) = refresh_interval() else {
        return;
    };
    let Some(marker) = refresh_marker(file) else {
        return;
    };
    let stamp = source_stamp(file);
    let fresh = std::fs::read_to_string(&marker)
        .ok()
        .map(|recorded| recorded == stamp)
        .unwrap_or(false)
        && std::fs::metadata(&marker)
            .and_then(|metadata| metadata.modified())
            .ok()
            .and_then(|modified| modified.elapsed().ok())
            .map(|age| age < interval)
            .unwrap_or(false);
    if fresh {
        return;
    }

    if let Some(parent) = marker.parent() {
        if std::fs::create_dir_all(parent).is_err() {
            return;
        }
    }
    let _ = std::fs::remove_file(&marker);
    let mut marker_file = match std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&marker)
    {
        Ok(file) => file,
        Err(_) => return,
    };
    if marker_file.write_all(stamp.as_bytes()).is_err() {
        let _ = std::fs::remove_file(&marker);
        return;
    }

    let executable = match env::current_exe() {
        Ok(path) => path,
        Err(_) => {
            let _ = std::fs::remove_file(&marker);
            return;
        }
    };
    if Command::new(executable)
        .args(["refresh", "--quiet", file])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .is_err()
    {
        let _ = std::fs::remove_file(marker);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_supported_tracker_links() {
        assert_eq!(
            tracker_ref("https://github.com/acme/widgets/pull/42?utm_source=scorecard"),
            Some(TrackerRef::GitHub {
                owner: "acme".into(),
                repo: "widgets".into(),
                number: 42,
            })
        );
        assert_eq!(
            tracker_ref("https://linear.app/acme/issue/ENG-123/fix-the-widget"),
            Some(TrackerRef::Linear {
                identifier: "ENG-123".into(),
            })
        );
        assert_eq!(
            tracker_ref("https://github.com/acme/widgets/issues/42"),
            None
        );
        assert_eq!(
            tracker_ref("https://example.com/acme/widgets/pull/42"),
            None
        );
    }

    #[test]
    fn tracked_rows_ignore_callouts_and_collect_all_criterion_links() {
        let source = "\
## Work
| A1 | risk | 3 | Ship | [PR](https://github.com/acme/widgets/pull/42) and [ENG-123](https://linear.app/acme/issue/ENG-123/fix) | grp:ship |
## Callouts
| NEXT | [another PR](https://github.com/acme/widgets/pull/99) |
";
        let rows = tracked_rows(source);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].refs.len(), 2);
        assert_eq!(rows[0].id, "A1");
    }

    #[test]
    fn multi_link_rows_require_every_tracker_to_be_terminal() {
        let github = TrackerRef::GitHub {
            owner: "acme".into(),
            repo: "widgets".into(),
            number: 42,
        };
        let linear = TrackerRef::Linear {
            identifier: "ENG-123".into(),
        };
        let row = TrackedRow {
            id: "A1".into(),
            refs: vec![github.clone(), linear.clone()],
        };
        let mut states = HashMap::new();
        states.insert(github, TrackerState::Terminal);
        assert!(!row_is_terminal(&row, &states));

        states.insert(linear.clone(), TrackerState::Open);
        assert!(!row_is_terminal(&row, &states));

        states.insert(linear, TrackerState::Terminal);
        assert!(row_is_terminal(&row, &states));
    }

    #[test]
    fn provider_states_have_conservative_terminal_semantics() {
        assert_eq!(github_state("MERGED"), TrackerState::Terminal);
        assert_eq!(github_state("CLOSED"), TrackerState::Terminal);
        assert_eq!(github_state("OPEN"), TrackerState::Open);
        assert_eq!(github_state("mystery"), TrackerState::Unknown);
        assert_eq!(linear_state("completed"), TrackerState::Terminal);
        assert_eq!(linear_state("canceled"), TrackerState::Terminal);
        assert_eq!(linear_state("started"), TrackerState::Open);
        assert_eq!(linear_state("unknown"), TrackerState::Unknown);
    }

    #[test]
    fn automatic_prune_is_structural_and_does_not_expand_groups() {
        let target = "|  A1|risk|3|one revised after lookup|[PR](https://github.com/acme/widgets/pull/42)|grp:ship|";
        let sibling =
            "| A2 | risk | 3 | two | [PR](https://github.com/acme/widgets/pull/43) | grp:ship |";
        let source = format!("## Work\n{}\n{}\n", target, sibling);
        let mut states = HashMap::new();
        states.insert(
            TrackerRef::GitHub {
                owner: "acme".into(),
                repo: "widgets".into(),
                number: 42,
            },
            TrackerState::Terminal,
        );
        let (after, removed) = prune_terminal_content(&source, &states);
        assert_eq!(removed, 1);
        assert!(!after.contains("|  A1|"));
        assert!(after.contains("| A2 |"));
        assert!(after.ends_with('\n'));
    }

    #[test]
    fn linear_credentials_stay_in_curl_stdin_config() {
        let config = linear_curl_config("secret-key", "query { issue(id:\"ENG-1\") { id } }");
        assert!(config.contains("Authorization: secret-key"));
        assert!(config.contains("https://api.linear.app/graphql"));
        assert!(config.contains("\\\\\\\"ENG-1\\\\\\\""));
    }
}
