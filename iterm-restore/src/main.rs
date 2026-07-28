//! iterm-restore — inventory open iTerm2 tabs and resume the Claude, Codex, or
//! Grok
//! session running in each one. Replaces the fragile
//! `bin/iterm-session-snapshot` zsh script with typed, testable logic.
//! Std-only: shells out to `osascript`, `ps`, `lsof`, `date` and parses their
//! text output by hand.
//!
//! Usage:
//!   iterm-restore                # default: resume sessions in place (already-open tabs)
//!   iterm-restore --list | -l    # print an inventory table, write nothing
//!   iterm-restore --new-window   # open one fresh window, one tab per session
//!   iterm-restore -o PATH        # write the emitted script to PATH
//!
//! Per-tab resolution (inference is the workhorse; the live path is a
//! best-effort bonus):
//!   1. live: a running agent on the tab's tty is caught holding its
//!      transcript open (`ps -t <tty>` then `lsof -p <pid>`). When it fires
//!      this is the exact session, even for tabs sharing a cwd. BUT it is
//!      unreliable: Claude opens the transcript only to append and closes it
//!      between writes, so a live-but-idle claude exposes no open fd, no
//!      session id in argv/env, and no lock file — there is no durable
//!      on-process signal to map it. Codex and Grok keep their session logs
//!      open, and Grok additionally records the exact PID-to-session mapping in
//!      `~/.grok/active_sessions.json`.
//!   2. inferred: the newest transcript whose cwd matches, ranked by
//!      in-content timestamp (file mtimes are unreliable after a
//!      migration/restore). This is what resolves most tabs in practice.
//!   3. ambiguous: several tabs share a cwd and none resolved live — the N
//!      most recent DISTINCT sessions are assigned across them, newest first,
//!      with the rest recorded as alternates. Which physical tab gets which
//!      of a cwd's sessions is arbitrary (same dir — rearrange by hand).
//!   4. shell: no resolvable transcript.
//!
//! The tool never resumes its own controlling tab (detected via $TTY, else
//! the tty of the tool's own process).

use std::collections::{HashMap, HashSet};
use std::env;
use std::fs;
use std::io::BufRead;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{exit, Command};

// ---------------------------------------------------------------------
// types
// ---------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Harness {
    Claude,
    Codex,
    Grok,
    Shell,
}

impl Harness {
    fn label(self) -> &'static str {
        match self {
            Harness::Claude => "claude",
            Harness::Codex => "codex",
            Harness::Grok => "grok",
            Harness::Shell => "shell",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Confidence {
    Live,
    Inferred,
    Ambiguous,
    None,
}

impl Confidence {
    fn label(self) -> &'static str {
        match self {
            Confidence::Live => "live",
            Confidence::Inferred => "inferred",
            Confidence::Ambiguous => "ambiguous",
            Confidence::None => "none",
        }
    }
}

#[derive(Debug, Clone)]
struct Tab {
    tty: String,
    cwd: String,
}

#[derive(Debug, Clone)]
struct Resolution {
    tty: String,
    cwd: String,
    harness: Harness,
    session_id: Option<String>,
    confidence: Confidence,
    is_self: bool,
    alternates: Vec<(Harness, String)>,
}

impl Resolution {
    fn shell(tab: &Tab) -> Resolution {
        Resolution {
            tty: tab.tty.clone(),
            cwd: tab.cwd.clone(),
            harness: Harness::Shell,
            session_id: None,
            confidence: Confidence::None,
            is_self: false,
            alternates: Vec::new(),
        }
    }
}

// ---------------------------------------------------------------------
// CLI
// ---------------------------------------------------------------------

#[derive(Debug, PartialEq, Eq)]
enum Mode {
    List,
    InPlace,
    NewWindow,
}

struct Args {
    mode: Mode,
    out: Option<String>,
}

fn parse_args(args: &[String]) -> Result<Args, String> {
    let mut mode = Mode::InPlace;
    let mut out = None;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--list" | "-l" => mode = Mode::List,
            "--in-place" => mode = Mode::InPlace,
            "--new-window" => mode = Mode::NewWindow,
            "-o" => {
                i += 1;
                let path = args.get(i).ok_or("-o requires a path")?;
                out = Some(path.clone());
            }
            other => return Err(format!("unrecognized argument: {other}")),
        }
        i += 1;
    }
    Ok(Args { mode, out })
}

// ---------------------------------------------------------------------
// iTerm tab enumeration
// ---------------------------------------------------------------------

const LIST_TABS_SCRIPT: &str = r#"tell application "iTerm2"
  set out to ""
  repeat with w in windows
    repeat with t in tabs of w
      repeat with s in sessions of t
        set p to ""
        try
          set p to (variable s named "session.path")
        end try
        set out to out & (tty of s) & "\t" & p & linefeed
      end repeat
    end repeat
  end repeat
  return out
end tell"#;

fn run_osascript(script: &str) -> String {
    Command::new("osascript")
        .arg("-e")
        .arg(script)
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .unwrap_or_default()
}

/// Parse the tab-list AppleScript's tty<TAB>cwd output. Rows with no tty are
/// dropped; rows with an empty cwd are kept (they still resolve, just not by
/// cwd — see the empty-cwd branch in `resolve_all`).
fn parse_tab_list(raw: &str) -> Vec<Tab> {
    raw.lines()
        .filter_map(|line| {
            let mut parts = line.splitn(2, '\t');
            let tty = parts.next()?.trim();
            if tty.is_empty() {
                return None;
            }
            let cwd = parts.next().unwrap_or("").trim().to_string();
            Some(Tab {
                tty: tty.to_string(),
                cwd,
            })
        })
        .collect()
}

/// The tty of this tool's own controlling terminal, so its own tab is never
/// targeted for resume. $TTY if set, else derived from this process's own
/// tty via `ps`.
fn own_tty() -> Option<String> {
    if let Ok(t) = env::var("TTY") {
        if !t.is_empty() {
            return Some(t);
        }
    }
    let pid = std::process::id().to_string();
    let output = Command::new("ps")
        .args(["-o", "tty=", "-p", &pid])
        .output()
        .ok()?;
    let s = String::from_utf8(output.stdout).ok()?;
    let dev = s.trim();
    if dev.is_empty() || dev == "??" {
        None
    } else {
        Some(format!("/dev/{dev}"))
    }
}

fn home_dir() -> PathBuf {
    PathBuf::from(env::var("HOME").expect("HOME not set"))
}

fn grok_home_dir() -> PathBuf {
    env::var_os("GROK_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| home_dir().join(".grok"))
}

/// Return complete top-level JSON objects while respecting quoted braces.
/// This is enough for Grok's flat active-session registry without adding a
/// JSON dependency to this otherwise std-only tool.
fn json_object_slices(input: &str) -> Vec<&str> {
    let mut objects = Vec::new();
    let mut start = None;
    let mut depth = 0usize;
    let mut in_string = false;
    let mut escaped = false;

    for (index, byte) in input.bytes().enumerate() {
        if in_string {
            if escaped {
                escaped = false;
            } else if byte == b'\\' {
                escaped = true;
            } else if byte == b'"' {
                in_string = false;
            }
            continue;
        }

        match byte {
            b'"' => in_string = true,
            b'{' => {
                if depth == 0 {
                    start = Some(index);
                }
                depth += 1;
            }
            b'}' if depth > 0 => {
                depth -= 1;
                if depth == 0 {
                    if let Some(object_start) = start.take() {
                        objects.push(&input[object_start..=index]);
                    }
                }
            }
            _ => {}
        }
    }

    objects
}

fn json_field_value<'a>(input: &'a str, key: &str) -> Option<&'a str> {
    let bytes = input.as_bytes();
    let mut index = 0usize;
    while index < bytes.len() {
        if bytes[index] != b'"' {
            index += 1;
            continue;
        }

        let string_start = index + 1;
        let mut string_end = string_start;
        let mut escaped = false;
        while string_end < bytes.len() {
            let byte = bytes[string_end];
            if escaped {
                escaped = false;
            } else if byte == b'\\' {
                escaped = true;
            } else if byte == b'"' {
                break;
            }
            string_end += 1;
        }
        if string_end == bytes.len() {
            return None;
        }

        if &input[string_start..string_end] == key {
            let mut colon = string_end + 1;
            while colon < bytes.len() && bytes[colon].is_ascii_whitespace() {
                colon += 1;
            }
            if colon < bytes.len() && bytes[colon] == b':' {
                return Some(input[colon + 1..].trim_start());
            }
        }
        index = string_end + 1;
    }
    None
}

fn json_string_field(input: &str, key: &str) -> Option<String> {
    let value = json_field_value(input, key)?;
    let mut chars = value.chars();
    if chars.next()? != '"' {
        return None;
    }

    let mut out = String::new();
    while let Some(ch) = chars.next() {
        match ch {
            '"' => return Some(out),
            '\\' => match chars.next()? {
                '"' => out.push('"'),
                '\\' => out.push('\\'),
                '/' => out.push('/'),
                'b' => out.push('\u{0008}'),
                'f' => out.push('\u{000c}'),
                'n' => out.push('\n'),
                'r' => out.push('\r'),
                't' => out.push('\t'),
                'u' => {
                    let digits: String = chars.by_ref().take(4).collect();
                    if digits.len() != 4 {
                        return None;
                    }
                    let codepoint = u32::from_str_radix(&digits, 16).ok()?;
                    out.push(char::from_u32(codepoint)?);
                }
                _ => return None,
            },
            _ => out.push(ch),
        }
    }
    None
}

fn json_u32_field(input: &str, key: &str) -> Option<u32> {
    let value = json_field_value(input, key)?;
    let digits: String = value.chars().take_while(|ch| ch.is_ascii_digit()).collect();
    if digits.is_empty() {
        None
    } else {
        digits.parse().ok()
    }
}

fn parse_grok_active_sessions(input: &str) -> HashMap<u32, String> {
    json_object_slices(input)
        .into_iter()
        .filter_map(|object| {
            let pid = json_u32_field(object, "pid")?;
            let session_id = json_string_field(object, "session_id")?;
            Some((pid, session_id))
        })
        .collect()
}

fn grok_active_sessions() -> HashMap<u32, String> {
    fs::read_to_string(grok_home_dir().join("active_sessions.json"))
        .map(|content| parse_grok_active_sessions(&content))
        .unwrap_or_default()
}

// ---------------------------------------------------------------------
// live path: ps on the tty, then lsof on candidate pids for an open
// transcript file
// ---------------------------------------------------------------------

fn run_ps_on_tty(shortdev: &str) -> String {
    Command::new("ps")
        .args(["-t", shortdev, "-o", "pid=,comm="])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .unwrap_or_default()
}

fn run_lsof(pid: u32) -> String {
    Command::new("lsof")
        .arg("-p")
        .arg(pid.to_string())
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .unwrap_or_default()
}

fn parse_ps_pid_comm(output: &str) -> Vec<(u32, String)> {
    output
        .lines()
        .filter_map(|line| {
            let mut parts = line.split_whitespace();
            let pid = parts.next()?.parse::<u32>().ok()?;
            let comm = parts.collect::<Vec<_>>().join(" ");
            if comm.is_empty() {
                None
            } else {
                Some((pid, comm))
            }
        })
        .collect()
}

/// PIDs on a tty whose command looks like it could be a Claude, Codex, or Grok
/// agent. Claude and some Codex distributions show up as a `node` process, so
/// match loosely and let the active registry or lsof disambiguate.
fn candidate_agent_pids(ps_output: &str) -> Vec<u32> {
    parse_ps_pid_comm(ps_output)
        .into_iter()
        .filter(|(_, comm)| {
            let lc = comm.to_lowercase();
            lc.contains("node")
                || lc.contains("claude")
                || lc.contains("codex")
                || lc.contains("grok")
        })
        .map(|(pid, _)| pid)
        .collect()
}

/// Scan `lsof -p <pid>` output for an open harness transcript file.
fn extract_transcript_from_lsof(lsof_output: &str) -> Option<(Harness, String)> {
    for line in lsof_output.lines() {
        let Some(field) = line.split_whitespace().last() else {
            continue;
        };
        if !field.ends_with(".jsonl") {
            continue;
        }
        if field.contains("/.claude/projects/") {
            return Some((Harness::Claude, field.to_string()));
        }
        if field.contains("/.codex/sessions/") {
            return Some((Harness::Codex, field.to_string()));
        }
        if field.contains("/.grok/sessions/")
            && (field.ends_with("/events.jsonl")
                || field.ends_with("/updates.jsonl")
                || field.ends_with("/chat_history.jsonl"))
        {
            return Some((Harness::Grok, field.to_string()));
        }
    }
    None
}

fn claude_id_from_path(path: &str) -> Option<String> {
    let file_name = Path::new(path).file_name()?.to_str()?;
    file_name.strip_suffix(".jsonl").map(|s| s.to_string())
}

fn codex_id_from_path(path: &str) -> Option<String> {
    let file_name = Path::new(path).file_name()?.to_str()?;
    parse_codex_filename(file_name).map(|(_, uuid)| uuid)
}

fn grok_id_from_path(path: &str) -> Option<String> {
    let transcript = Path::new(path);
    match transcript.file_name()?.to_str()? {
        "events.jsonl" | "updates.jsonl" | "chat_history.jsonl" => {}
        _ => return None,
    }
    transcript
        .parent()?
        .file_name()?
        .to_str()
        .map(str::to_string)
}

fn try_live_resolution(
    tab: &Tab,
    active_grok_sessions: &HashMap<u32, String>,
) -> Option<Resolution> {
    let shortdev = tab.tty.strip_prefix("/dev/").unwrap_or(tab.tty.as_str());
    let ps_out = run_ps_on_tty(shortdev);
    for pid in candidate_agent_pids(&ps_out) {
        if let Some(id) = active_grok_sessions.get(&pid) {
            return Some(Resolution {
                tty: tab.tty.clone(),
                cwd: tab.cwd.clone(),
                harness: Harness::Grok,
                session_id: Some(id.clone()),
                confidence: Confidence::Live,
                is_self: false,
                alternates: Vec::new(),
            });
        }

        let lsof_out = run_lsof(pid);
        if let Some((harness, path)) = extract_transcript_from_lsof(&lsof_out) {
            let id = match harness {
                Harness::Claude => claude_id_from_path(&path),
                Harness::Codex => codex_id_from_path(&path),
                Harness::Grok => grok_id_from_path(&path),
                Harness::Shell => None,
            };
            if let Some(id) = id {
                return Some(Resolution {
                    tty: tab.tty.clone(),
                    cwd: tab.cwd.clone(),
                    harness,
                    session_id: Some(id),
                    confidence: Confidence::Live,
                    is_self: false,
                    alternates: Vec::new(),
                });
            }
        }
    }
    None
}

// ---------------------------------------------------------------------
// dead path: infer from transcripts on disk
// ---------------------------------------------------------------------

/// /Users/rch/repo/occult -> -Users-rch-repo-occult
fn cwd_to_mangled(cwd: &str) -> String {
    cwd.replace('/', "-")
}

/// rollout-2026-07-23T20-24-45-019f9227-6f06-7c31-bb22-f9046c5aab9e.jsonl
/// -> ("2026-07-23T20-24-45", "019f9227-6f06-7c31-bb22-f9046c5aab9e")
///
/// The timestamp prefix is a fixed-width 19 chars (YYYY-MM-DDTHH-MM-SS); the
/// uuid is taken as everything after it, so a naive split on '-' can't
/// truncate the uuid at its own internal hyphens.
fn parse_codex_filename(filename: &str) -> Option<(String, String)> {
    let stem = filename.strip_suffix(".jsonl")?;
    let rest = stem.strip_prefix("rollout-")?;
    if rest.len() < 20 {
        return None;
    }
    let (ts, rest2) = rest.split_at(19);
    let uuid = rest2.strip_prefix('-')?;
    if uuid.is_empty() {
        return None;
    }
    Some((ts.to_string(), uuid.to_string()))
}

/// codex records `"cwd":"<path>"` on line 1 of a rollout. Anchor on the
/// trailing quote so `/x/occult` does not match a rollout recording
/// `/x/occult-wt-correctness`.
fn codex_line_matches_cwd(line: &str, cwd: &str) -> bool {
    let needle = format!("\"cwd\":\"{cwd}\"");
    line.contains(&needle)
}

fn extract_timestamp(line: &str) -> Option<String> {
    let key = "\"timestamp\":\"";
    let start = line.find(key)? + key.len();
    let end = line[start..].find('"')? + start;
    Some(line[start..end].to_string())
}

/// Last-activity timestamp for a claude transcript: the timestamp of the last
/// line that HAS one (file mtimes are unreliable; a naive first-line read
/// would report the session's start time, not its last activity). Claude
/// transcripts routinely end with a `{"type":"last-prompt"}` (and other)
/// trailer records that carry no timestamp, so we scan backward past them —
/// reading only the final line would drop such a session from ranking
/// entirely. Timestamps are monotonic through the file, so the last one that
/// parses is the true last activity.
fn last_activity_timestamp(jsonl_content: &str) -> Option<String> {
    jsonl_content.lines().rev().find_map(extract_timestamp)
}

fn first_line(path: &Path) -> Option<String> {
    let file = fs::File::open(path).ok()?;
    let mut reader = std::io::BufReader::new(file);
    let mut line = String::new();
    reader.read_line(&mut line).ok()?;
    Some(line)
}

fn claude_candidates_for_cwd(cwd: &str) -> Vec<(String, String)> {
    let dir = home_dir()
        .join(".claude/projects")
        .join(cwd_to_mangled(cwd));
    let mut out = Vec::new();
    let Ok(entries) = fs::read_dir(&dir) else {
        return out;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("jsonl") {
            continue;
        }
        let Some(file_name) = path.file_name().and_then(|f| f.to_str()) else {
            continue;
        };
        let Some(id) = file_name.strip_suffix(".jsonl") else {
            continue;
        };
        let Ok(content) = fs::read_to_string(&path) else {
            continue;
        };
        if let Some(ts) = last_activity_timestamp(&content) {
            out.push((id.to_string(), ts));
        }
    }
    out
}

fn codex_candidates_for_cwd(cwd: &str) -> Vec<(String, String)> {
    let root = home_dir().join(".codex/sessions");
    let mut out = Vec::new();
    walk_codex_dir(&root, cwd, &mut out);
    out
}

fn walk_codex_dir(dir: &Path, cwd: &str, out: &mut Vec<(String, String)>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            walk_codex_dir(&path, cwd, out);
            continue;
        }
        let Some(file_name) = path.file_name().and_then(|f| f.to_str()) else {
            continue;
        };
        if !file_name.starts_with("rollout-") || !file_name.ends_with(".jsonl") {
            continue;
        }
        let Some((ts, uuid)) = parse_codex_filename(file_name) else {
            continue;
        };
        let Some(line) = first_line(&path) else {
            continue;
        };
        if codex_line_matches_cwd(&line, cwd) {
            out.push((uuid, ts));
        }
    }
}

fn percent_encode_path_component(path: &str) -> String {
    const HEX: &[u8; 16] = b"0123456789ABCDEF";
    let mut encoded = String::with_capacity(path.len());
    for byte in path.bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'.' | b'_' | b'~') {
            encoded.push(char::from(byte));
        } else {
            encoded.push('%');
            encoded.push(char::from(HEX[(byte >> 4) as usize]));
            encoded.push(char::from(HEX[(byte & 0x0f) as usize]));
        }
    }
    encoded
}

fn grok_session_groups_for_cwd(root: &Path, cwd: &str) -> Vec<PathBuf> {
    let mut groups = Vec::new();
    let direct = root.join(percent_encode_path_component(cwd));
    if direct.is_dir() {
        groups.push(direct.clone());
    }

    let Ok(entries) = fs::read_dir(root) else {
        return groups;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path == direct || !path.is_dir() {
            continue;
        }
        let Ok(recorded_cwd) = fs::read_to_string(path.join(".cwd")) else {
            continue;
        };
        if recorded_cwd.trim_end() == cwd {
            groups.push(path);
        }
    }
    groups
}

fn grok_candidates_in_root(root: &Path, cwd: &str) -> Vec<(String, String)> {
    let mut out = Vec::new();
    for group in grok_session_groups_for_cwd(root, cwd) {
        let Ok(entries) = fs::read_dir(group) else {
            continue;
        };
        for entry in entries.flatten() {
            let session_dir = entry.path();
            if !session_dir.is_dir() {
                continue;
            }
            let Some(id) = session_dir
                .file_name()
                .and_then(|name| name.to_str())
                .map(str::to_string)
            else {
                continue;
            };
            let Ok(summary) = fs::read_to_string(session_dir.join("summary.json")) else {
                continue;
            };
            let timestamp = json_string_field(&summary, "updated_at")
                .or_else(|| json_string_field(&summary, "last_active_at"))
                .or_else(|| json_string_field(&summary, "created_at"));
            if let Some(timestamp) = timestamp {
                out.push((id, timestamp));
            }
        }
    }
    out
}

fn grok_candidates_for_cwd(cwd: &str) -> Vec<(String, String)> {
    grok_candidates_in_root(&grok_home_dir().join("sessions"), cwd)
}

// ---------------------------------------------------------------------
// ranking / duplicate-cwd assignment
// ---------------------------------------------------------------------

/// Claude timestamps are ISO-8601 with colons and milliseconds
/// (2026-07-24T05:48:30.612Z); codex filename timestamps use hyphens and no
/// sub-second precision (2026-07-23T20-24-45). Neither sorts correctly
/// against the other as raw text. Strip to digits-only and pad on the right
/// so a missing sub-second component compares as :00.000 rather than as a
/// short (and therefore lexically small) string.
fn normalize_ts(ts: &str) -> String {
    let digits: String = ts.chars().filter(|c| c.is_ascii_digit()).collect();
    format!("{digits:0<20}")
}

struct Candidate {
    harness: Harness,
    id: String,
    rank_key: String,
}

struct Assignment {
    harness: Harness,
    id: Option<String>,
    confidence: Confidence,
    alternates: Vec<(Harness, String)>,
}

fn remove_claimed_candidates(
    candidates: &mut Vec<Candidate>,
    claimed_sessions: &HashSet<(Harness, String)>,
) {
    candidates
        .retain(|candidate| !claimed_sessions.contains(&(candidate.harness, candidate.id.clone())));
}

/// Assign the `tab_count` most-recent DISTINCT candidates across `tab_count`
/// tabs sharing a cwd, newest first. If there are fewer candidates than
/// tabs, the extra tabs get no session (shell). Every assigned tab is
/// flagged ambiguous with the other candidates recorded as alternates.
fn assign_duplicate_cwd_sessions(
    tab_count: usize,
    mut candidates: Vec<Candidate>,
) -> Vec<Assignment> {
    candidates.sort_by(|a, b| b.rank_key.cmp(&a.rank_key));
    let mut out = Vec::with_capacity(tab_count);
    for i in 0..tab_count {
        match candidates.get(i) {
            Some(cand) => {
                let alternates = candidates
                    .iter()
                    .enumerate()
                    .filter(|(j, _)| *j != i)
                    .map(|(_, c)| (c.harness, c.id.clone()))
                    .collect();
                out.push(Assignment {
                    harness: cand.harness,
                    id: Some(cand.id.clone()),
                    confidence: Confidence::Ambiguous,
                    alternates,
                });
            }
            None => out.push(Assignment {
                harness: Harness::Shell,
                id: None,
                confidence: Confidence::None,
                alternates: Vec::new(),
            }),
        }
    }
    out
}

// ---------------------------------------------------------------------
// resolver orchestration
// ---------------------------------------------------------------------

fn resolve_all(tabs: &[Tab], self_tty: Option<&str>) -> Vec<Resolution> {
    let mut by_tty: HashMap<String, Resolution> = HashMap::new();
    let mut dead: Vec<&Tab> = Vec::new();
    let mut claimed_sessions: HashSet<(Harness, String)> = HashSet::new();
    let active_grok_sessions = grok_active_sessions();

    for tab in tabs {
        match try_live_resolution(tab, &active_grok_sessions) {
            Some(res) => {
                if let Some(id) = &res.session_id {
                    claimed_sessions.insert((res.harness, id.clone()));
                }
                by_tty.insert(tab.tty.clone(), res);
            }
            None => dead.push(tab),
        }
    }

    let mut groups: HashMap<String, Vec<&Tab>> = HashMap::new();
    for tab in dead {
        groups.entry(tab.cwd.clone()).or_default().push(tab);
    }

    for (cwd, group) in groups {
        if cwd.is_empty() {
            for tab in group {
                by_tty.insert(tab.tty.clone(), Resolution::shell(tab));
            }
            continue;
        }

        let mut candidates: Vec<Candidate> = Vec::new();
        for (id, ts) in claude_candidates_for_cwd(&cwd) {
            candidates.push(Candidate {
                harness: Harness::Claude,
                id,
                rank_key: normalize_ts(&ts),
            });
        }
        for (id, ts) in codex_candidates_for_cwd(&cwd) {
            candidates.push(Candidate {
                harness: Harness::Codex,
                id,
                rank_key: normalize_ts(&ts),
            });
        }
        for (id, ts) in grok_candidates_for_cwd(&cwd) {
            candidates.push(Candidate {
                harness: Harness::Grok,
                id,
                rank_key: normalize_ts(&ts),
            });
        }
        remove_claimed_candidates(&mut candidates, &claimed_sessions);

        if group.len() == 1 {
            let tab = group[0];
            candidates.sort_by(|a, b| b.rank_key.cmp(&a.rank_key));
            let res = match candidates.first() {
                Some(c) => Resolution {
                    tty: tab.tty.clone(),
                    cwd: cwd.clone(),
                    harness: c.harness,
                    session_id: Some(c.id.clone()),
                    confidence: Confidence::Inferred,
                    is_self: false,
                    alternates: Vec::new(),
                },
                None => Resolution::shell(tab),
            };
            by_tty.insert(tab.tty.clone(), res);
        } else {
            let assignments = assign_duplicate_cwd_sessions(group.len(), candidates);
            for (tab, assignment) in group.iter().zip(assignments) {
                by_tty.insert(
                    tab.tty.clone(),
                    Resolution {
                        tty: tab.tty.clone(),
                        cwd: cwd.clone(),
                        harness: assignment.harness,
                        session_id: assignment.id,
                        confidence: assignment.confidence,
                        is_self: false,
                        alternates: assignment.alternates,
                    },
                );
            }
        }
    }

    tabs.iter()
        .map(|t| {
            let mut res = by_tty
                .remove(&t.tty)
                .unwrap_or_else(|| Resolution::shell(t));
            if Some(t.tty.as_str()) == self_tty {
                res.is_self = true;
            }
            res
        })
        .collect()
}

// ---------------------------------------------------------------------
// output: --list
// ---------------------------------------------------------------------

fn print_list(resolutions: &[Resolution]) {
    println!(
        "{:<16} {:<45} {:<8} {:<38} CONFIDENCE",
        "TTY", "CWD", "HARNESS", "SESSION-ID"
    );
    for r in resolutions {
        let id = r.session_id.as_deref().unwrap_or("-");
        let conf = if r.is_self {
            "self"
        } else {
            r.confidence.label()
        };
        println!(
            "{:<16} {:<45} {:<8} {:<38} {}",
            r.tty,
            r.cwd,
            r.harness.label(),
            id,
            conf
        );
    }
}

// ---------------------------------------------------------------------
// output: script generation
// ---------------------------------------------------------------------

fn resume_command(harness: Harness, session_id: &Option<String>) -> Option<String> {
    match (harness, session_id) {
        (Harness::Claude, Some(id)) => Some(format!("claude --resume {id}")),
        (Harness::Codex, Some(id)) => Some(format!("codex resume {id}")),
        (Harness::Grok, Some(id)) => Some(format!("grok --resume {id}")),
        _ => None,
    }
}

fn escape_applescript(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

/// Cap on alternates shown in a script header comment. A cwd with a long
/// history (many past sessions) can have hundreds of candidates; the
/// assignment logic considers all of them, but the comment only needs enough
/// to review the top few by hand.
const MAX_ALTERNATES_SHOWN: usize = 5;

fn alternates_comment(r: &Resolution) -> Option<String> {
    if r.confidence != Confidence::Ambiguous || r.alternates.is_empty() {
        return None;
    }
    let mut list = r
        .alternates
        .iter()
        .take(MAX_ALTERNATES_SHOWN)
        .map(|(h, id)| format!("{}:{}", h.label(), id))
        .collect::<Vec<_>>()
        .join(", ");
    if r.alternates.len() > MAX_ALTERNATES_SHOWN {
        list.push_str(&format!(
            " (+{} more)",
            r.alternates.len() - MAX_ALTERNATES_SHOWN
        ));
    }
    Some(format!("# {} ambiguous — alternates: {}\n", r.tty, list))
}

fn build_in_place_script(resolutions: &[Resolution]) -> String {
    let mut body = String::new();
    body.push_str("#!/usr/bin/env zsh\n");
    body.push_str("# Auto-generated by iterm-restore. Resumes agent sessions in place,\n");
    body.push_str("# in the iTerm tabs that are already open.\n");
    for r in resolutions {
        if let Some(c) = alternates_comment(r) {
            body.push_str(&c);
        }
    }
    body.push_str("set -eu\n");
    body.push_str("osascript <<'APPLESCRIPT'\n");
    body.push_str("tell application \"iTerm2\"\n");
    body.push_str("  repeat with w in windows\n");
    body.push_str("    repeat with t in tabs of w\n");
    body.push_str("      repeat with s in sessions of t\n");
    body.push_str("        set theTty to (tty of s)\n");
    let mut first = true;
    let mut any = false;
    for r in resolutions {
        if r.is_self {
            continue;
        }
        let Some(cmd) = resume_command(r.harness, &r.session_id) else {
            continue;
        };
        any = true;
        let branch = if first { "if" } else { "else if" };
        first = false;
        body.push_str(&format!("        {branch} theTty is \"{}\" then\n", r.tty));
        body.push_str(&format!(
            "          tell s to write text \"{}\"\n",
            escape_applescript(&cmd)
        ));
    }
    if any {
        body.push_str("        end if\n");
    }
    body.push_str("      end repeat\n");
    body.push_str("    end repeat\n");
    body.push_str("  end repeat\n");
    body.push_str("end tell\n");
    body.push_str("APPLESCRIPT\n");
    body
}

fn build_new_window_script(resolutions: &[Resolution]) -> String {
    let mut body = String::new();
    body.push_str("#!/usr/bin/env zsh\n");
    body.push_str("# Auto-generated by iterm-restore. Opens a new iTerm window with one\n");
    body.push_str("# tab per captured session.\n");
    for r in resolutions {
        if let Some(c) = alternates_comment(r) {
            body.push_str(&c);
        }
    }
    body.push_str("set -eu\n");
    body.push_str("osascript <<'APPLESCRIPT'\n");
    body.push_str("tell application \"iTerm2\"\n");
    body.push_str("  set newWin to (create window with default profile)\n");
    let mut first = true;
    for r in resolutions {
        if r.is_self {
            continue;
        }
        let raw_line = match resume_command(r.harness, &r.session_id) {
            Some(cmd) => format!("cd \"{}\" && {}", r.cwd, cmd),
            None => format!("cd \"{}\"", r.cwd),
        };
        let escaped = escape_applescript(&raw_line);
        if first {
            body.push_str(&format!(
                "  tell current session of newWin to write text \"{escaped}\"\n"
            ));
            first = false;
        } else {
            body.push_str("  tell newWin\n");
            body.push_str("    set newTab to (create tab with default profile)\n");
            body.push_str(&format!(
                "    tell current session of newTab to write text \"{escaped}\"\n"
            ));
            body.push_str("  end tell\n");
        }
    }
    body.push_str("end tell\n");
    body.push_str("APPLESCRIPT\n");
    body
}

fn default_output_path() -> String {
    let ts = Command::new("date")
        .arg("+%Y%m%d-%H%M%S")
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "unknown".to_string());
    format!("./iterm-restore-{ts}.sh")
}

fn write_script(path: &str, content: &str) -> std::io::Result<()> {
    fs::write(path, content)?;
    let mut perms = fs::metadata(path)?.permissions();
    perms.set_mode(0o755);
    fs::set_permissions(path, perms)
}

// ---------------------------------------------------------------------
// main
// ---------------------------------------------------------------------

fn main() {
    let raw_args: Vec<String> = env::args().skip(1).collect();
    let args = match parse_args(&raw_args) {
        Ok(a) => a,
        Err(e) => {
            eprintln!("iterm-restore: {e}");
            exit(2);
        }
    };

    if Command::new("which")
        .arg("osascript")
        .output()
        .map(|o| !o.status.success())
        .unwrap_or(true)
    {
        eprintln!("iterm-restore: osascript required (macOS + iTerm2)");
        exit(1);
    }

    let tab_list_raw = run_osascript(LIST_TABS_SCRIPT);
    let tabs = parse_tab_list(&tab_list_raw);
    if tabs.is_empty() {
        eprintln!("iterm-restore: no iTerm sessions found (is iTerm2 running?)");
        exit(1);
    }

    let self_tty = own_tty();
    let resolutions = resolve_all(&tabs, self_tty.as_deref());

    match args.mode {
        Mode::List => print_list(&resolutions),
        Mode::InPlace => {
            let script = build_in_place_script(&resolutions);
            let path = args.out.unwrap_or_else(default_output_path);
            if let Err(e) = write_script(&path, &script) {
                eprintln!("iterm-restore: failed to write {path}: {e}");
                exit(1);
            }
            println!("wrote restore script: {path}");
            println!("review it, then run: {path}");
        }
        Mode::NewWindow => {
            let script = build_new_window_script(&resolutions);
            let path = args.out.unwrap_or_else(default_output_path);
            if let Err(e) = write_script(&path, &script) {
                eprintln!("iterm-restore: failed to write {path}: {e}");
                exit(1);
            }
            println!("wrote restore script: {path}");
            println!("review it, then run: {path}");
        }
    }
}

// ---------------------------------------------------------------------
// tests
// ---------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn test_temp_dir(label: &str) -> PathBuf {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        env::temp_dir().join(format!(
            "iterm-restore-{label}-{}-{unique}",
            std::process::id()
        ))
    }

    // codex filename -> (timestamp, uuid). Failure hypothesis: a naive split
    // on '-' truncates the uuid at its own internal hyphens.
    #[test]
    fn codex_filename_parses_uuid_with_hyphens_intact() {
        let name = "rollout-2026-07-23T20-24-45-019f9227-6f06-7c31-bb22-f9046c5aab9e.jsonl";
        let (ts, uuid) = parse_codex_filename(name).unwrap();
        assert_eq!(ts, "2026-07-23T20-24-45");
        assert_eq!(uuid, "019f9227-6f06-7c31-bb22-f9046c5aab9e");
    }

    #[test]
    fn codex_filename_rejects_non_rollout_names() {
        assert!(parse_codex_filename("not-a-rollout.jsonl").is_none());
        assert!(parse_codex_filename("rollout-too-short.jsonl").is_none());
    }

    // cwd -> mangled claude project dir name.
    #[test]
    fn cwd_to_mangled_replaces_all_slashes() {
        assert_eq!(
            cwd_to_mangled("/Users/rch/repo/occult"),
            "-Users-rch-repo-occult"
        );
    }

    #[test]
    fn grok_cwd_encoding_matches_session_directory_names() {
        assert_eq!(
            percent_encode_path_component("/Users/rch/repo/occult"),
            "%2FUsers%2Frch%2Frepo%2Foccult"
        );
        assert_eq!(
            percent_encode_path_component("/Users/rch/repo/a b#c"),
            "%2FUsers%2Frch%2Frepo%2Fa%20b%23c"
        );
    }

    #[test]
    fn grok_candidates_find_encoded_and_long_path_groups() {
        let root = test_temp_dir("grok-candidates");
        let cwd = "/Users/rch/repo/a b";
        let direct_group = root.join(percent_encode_path_component(cwd));
        let direct_session = direct_group.join("019f0000-0000-7000-8000-000000000001");
        fs::create_dir_all(&direct_session).unwrap();
        fs::write(
            direct_session.join("summary.json"),
            r#"{"updated_at":"2026-07-27T20:00:00.000Z"}"#,
        )
        .unwrap();

        let long_group = root.join("long-path-deadbeef");
        let long_session = long_group.join("019f0000-0000-7000-8000-000000000002");
        fs::create_dir_all(&long_session).unwrap();
        fs::write(long_group.join(".cwd"), format!("{cwd}\n")).unwrap();
        fs::write(
            long_session.join("summary.json"),
            r#"{"last_active_at":"2026-07-27T21:00:00.000Z"}"#,
        )
        .unwrap();

        let mut candidates = grok_candidates_in_root(&root, cwd);
        candidates.sort();
        assert_eq!(
            candidates,
            vec![
                (
                    "019f0000-0000-7000-8000-000000000001".to_string(),
                    "2026-07-27T20:00:00.000Z".to_string()
                ),
                (
                    "019f0000-0000-7000-8000-000000000002".to_string(),
                    "2026-07-27T21:00:00.000Z".to_string()
                ),
            ]
        );

        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn grok_active_registry_maps_pid_to_session_id() {
        let registry = r#"[
          {
            "session_id": "019f0000-0000-7000-8000-000000000001",
            "pid": 1234,
            "cwd": "/tmp/a {quoted} directory"
          },
          {
            "session_id": "019f0000-0000-7000-8000-000000000002",
            "pid": 5678,
            "cwd": "/tmp/b"
          }
        ]"#;
        let active = parse_grok_active_sessions(registry);
        assert_eq!(
            active.get(&1234).map(String::as_str),
            Some("019f0000-0000-7000-8000-000000000001")
        );
        assert_eq!(
            active.get(&5678).map(String::as_str),
            Some("019f0000-0000-7000-8000-000000000002")
        );
    }

    #[test]
    fn json_field_lookup_ignores_key_text_inside_string_values() {
        let summary = r#"{
          "session_summary": "mentions \"updated_at\": \"not-a-field\"",
          "updated_at": "2026-07-27T22:00:00.000Z"
        }"#;
        assert_eq!(
            json_string_field(summary, "updated_at"),
            Some("2026-07-27T22:00:00.000Z".to_string())
        );
    }

    // codex cwd match must be trailing-quote-anchored. Failure hypothesis: a
    // plain substring match on `"cwd":"/x/occult` would also match a rollout
    // recording `/x/occult-wt-correctness`.
    #[test]
    fn codex_cwd_match_is_trailing_quote_anchored() {
        let cwd = "/Users/rch/repo/occult";
        let matching_line = "{\"cwd\":\"/Users/rch/repo/occult\",\"other\":1}";
        let non_matching_line = "{\"cwd\":\"/Users/rch/repo/occult-wt-correctness\"}";
        assert!(codex_line_matches_cwd(matching_line, cwd));
        assert!(!codex_line_matches_cwd(non_matching_line, cwd));
    }

    // claude last-activity extraction from a multi-line JSONL fixture: the
    // LAST line's timestamp wins, not the first.
    #[test]
    fn claude_last_activity_uses_final_line_timestamp() {
        let jsonl = "{\"type\":\"summary\"}\n\
             {\"timestamp\":\"2026-07-20T10:00:00.000Z\",\"type\":\"user\"}\n\
             {\"timestamp\":\"2026-07-24T05:48:30.612Z\",\"type\":\"assistant\"}\n";
        assert_eq!(
            last_activity_timestamp(jsonl),
            Some("2026-07-24T05:48:30.612Z".to_string())
        );
    }

    #[test]
    fn claude_last_activity_ignores_trailing_blank_lines() {
        let jsonl = "{\"timestamp\":\"2026-07-24T05:48:30.612Z\"}\n\n\n";
        assert_eq!(
            last_activity_timestamp(jsonl),
            Some("2026-07-24T05:48:30.612Z".to_string())
        );
    }

    // Real claude transcripts end with trailer records that carry no
    // timestamp (observed: a final {"type":"last-prompt"} line, sometimes
    // preceded by an untimestamped system line). Failure hypothesis: reading
    // only the final line yields None, dropping a cleanly-ended session from
    // ranking so an older/other-harness session wins (observed: latchkey
    // resolved to a 13:18 codex instead of its 15:52 claude).
    #[test]
    fn claude_last_activity_scans_past_untimestamped_trailer_records() {
        let jsonl = "{\"timestamp\":\"2026-07-23T15:52:35.947Z\",\"type\":\"message\"}\n\
             {\"type\":\"system\",\"content\":\"x\"}\n\
             {\"type\":\"last-prompt\"}\n";
        assert_eq!(
            last_activity_timestamp(jsonl),
            Some("2026-07-23T15:52:35.947Z".to_string())
        );
    }

    // duplicate-cwd assignment: 2 tabs, 3 candidates -> the 2 tabs get the 2
    // most-recent DISTINCT sessions, none shared, both ambiguous, alternates
    // populated with the rest.
    #[test]
    fn assign_duplicate_cwd_picks_most_recent_distinct_and_flags_ambiguous() {
        let candidates = vec![
            Candidate {
                harness: Harness::Claude,
                id: "aaa".to_string(),
                rank_key: normalize_ts("2026-01-01T00:00:00.000Z"),
            },
            Candidate {
                harness: Harness::Codex,
                id: "bbb".to_string(),
                rank_key: normalize_ts("2026-03-01T00-00-00"), // most recent
            },
            Candidate {
                harness: Harness::Claude,
                id: "ccc".to_string(),
                rank_key: normalize_ts("2026-02-01T00:00:00.000Z"),
            },
        ];
        let out = assign_duplicate_cwd_sessions(2, candidates);
        assert_eq!(out.len(), 2);

        assert_eq!(out[0].id.as_deref(), Some("bbb"));
        assert_eq!(out[1].id.as_deref(), Some("ccc"));
        assert_ne!(out[0].id, out[1].id);
        assert_eq!(out[0].confidence, Confidence::Ambiguous);
        assert_eq!(out[1].confidence, Confidence::Ambiguous);

        // alternates exclude self, include the other two candidates
        assert_eq!(out[0].alternates.len(), 2);
        assert!(out[0].alternates.iter().any(|(_, id)| id == "ccc"));
        assert!(out[0].alternates.iter().any(|(_, id)| id == "aaa"));
        assert!(!out[0].alternates.iter().any(|(_, id)| id == "bbb"));
    }

    #[test]
    fn assign_duplicate_cwd_leaves_extra_tabs_as_shell_when_candidates_run_out() {
        let candidates = vec![Candidate {
            harness: Harness::Claude,
            id: "only-one".to_string(),
            rank_key: normalize_ts("2026-01-01T00:00:00.000Z"),
        }];
        let out = assign_duplicate_cwd_sessions(2, candidates);
        assert_eq!(out[0].id.as_deref(), Some("only-one"));
        assert_eq!(out[1].id, None);
        assert_eq!(out[1].harness, Harness::Shell);
        assert_eq!(out[1].confidence, Confidence::None);
    }

    #[test]
    fn claimed_live_sessions_are_not_reused_for_inference() {
        let mut candidates = vec![
            Candidate {
                harness: Harness::Grok,
                id: "live-grok".to_string(),
                rank_key: normalize_ts("2026-07-27T22:00:00.000Z"),
            },
            Candidate {
                harness: Harness::Claude,
                id: "available-claude".to_string(),
                rank_key: normalize_ts("2026-07-27T21:00:00.000Z"),
            },
        ];
        let claimed = HashSet::from([(Harness::Grok, "live-grok".to_string())]);

        remove_claimed_candidates(&mut candidates, &claimed);

        assert_eq!(candidates.len(), 1);
        assert_eq!(candidates[0].harness, Harness::Claude);
        assert_eq!(candidates[0].id, "available-claude");
    }

    // normalize_ts: a claude ISO timestamp (colons + ms) and a codex
    // filename timestamp (hyphens, no ms) must compare correctly against
    // each other, not just within their own format.
    #[test]
    fn normalize_ts_orders_mixed_claude_and_codex_formats() {
        let claude_earlier = normalize_ts("2026-07-23T20-24-45"); // codex-style, earlier
        let claude_later = normalize_ts("2026-07-24T05:48:30.612Z"); // claude-style, later
        assert!(claude_later > claude_earlier);
    }

    #[test]
    fn parse_tab_list_splits_tty_and_cwd_and_drops_empty_tty_rows() {
        let raw = "/dev/ttys003\t/Users/rch/repo/occult\n\t/ignored\n/dev/ttys007\t\n";
        let tabs = parse_tab_list(raw);
        assert_eq!(tabs.len(), 2);
        assert_eq!(tabs[0].tty, "/dev/ttys003");
        assert_eq!(tabs[0].cwd, "/Users/rch/repo/occult");
        assert_eq!(tabs[1].tty, "/dev/ttys007");
        assert_eq!(tabs[1].cwd, "");
    }

    #[test]
    fn extract_transcript_from_lsof_finds_claude_codex_and_grok_paths() {
        let claude_lsof = "node    1234 rch  txt REG 1,4  1234 5678 /Users/rch/.claude/projects/-Users-rch-repo-occult/019f9227-abcd.jsonl";
        let codex_lsof = "node    5678 rch  txt REG 1,4  1234 5678 /Users/rch/.codex/sessions/2026/07/rollout-2026-07-23T20-24-45-019f9227-6f06-7c31-bb22-f9046c5aab9e.jsonl";
        let grok_lsof = "grok    9012 rch  27w REG 1,4 1234 5678 /Users/rch/.grok/sessions/%2FUsers%2Frch%2Frepo%2Foccult/019f9227-6f06-7c31-bb22-f9046c5aab9e/events.jsonl";
        let other_lsof = "node    9999 rch  txt REG 1,4  1234 5678 /Users/rch/somewhere/notes.txt";

        assert_eq!(
            extract_transcript_from_lsof(claude_lsof),
            Some((
                Harness::Claude,
                "/Users/rch/.claude/projects/-Users-rch-repo-occult/019f9227-abcd.jsonl"
                    .to_string()
            ))
        );
        assert_eq!(
            extract_transcript_from_lsof(codex_lsof).map(|(h, _)| h),
            Some(Harness::Codex)
        );
        assert_eq!(
            extract_transcript_from_lsof(grok_lsof).map(|(h, _)| h),
            Some(Harness::Grok)
        );
        assert_eq!(extract_transcript_from_lsof(other_lsof), None);
    }

    #[test]
    fn claude_id_from_path_strips_jsonl_suffix() {
        assert_eq!(
            claude_id_from_path("/Users/rch/.claude/projects/-x/019f9227-abcd.jsonl"),
            Some("019f9227-abcd".to_string())
        );
    }

    #[test]
    fn grok_id_from_path_uses_session_directory() {
        assert_eq!(
            grok_id_from_path(
                "/Users/rch/.grok/sessions/%2FUsers%2Frch%2Frepo%2Foccult/019f9227-6f06-7c31-bb22-f9046c5aab9e/events.jsonl"
            ),
            Some("019f9227-6f06-7c31-bb22-f9046c5aab9e".to_string())
        );
        assert_eq!(
            grok_id_from_path("/Users/rch/.grok/sessions/x/session/summary.json"),
            None
        );
    }

    #[test]
    fn candidate_agent_pids_matches_all_harnesses_case_insensitively() {
        let ps = "  1234 node\n  5678 zsh\n  9012 Codex\n  3456 -zsh\n  7890 Grok\n";
        let pids = candidate_agent_pids(ps);
        assert_eq!(pids, vec![1234, 9012, 7890]);
    }

    #[test]
    fn grok_resume_command_uses_resume_flag() {
        assert_eq!(
            resume_command(
                Harness::Grok,
                &Some("019f9227-6f06-7c31-bb22-f9046c5aab9e".to_string())
            ),
            Some("grok --resume 019f9227-6f06-7c31-bb22-f9046c5aab9e".to_string())
        );
    }

    // alternates_comment: a cwd with a long session history can produce
    // hundreds of candidates. Failure hypothesis: an unbounded join would
    // make the emitted script's header comment unreadable (observed: a real
    // repo directory produced a single comment line with 300+ entries).
    #[test]
    fn alternates_comment_truncates_long_candidate_lists() {
        let alternates: Vec<(Harness, String)> = (0..20)
            .map(|i| (Harness::Codex, format!("id-{i}")))
            .collect();
        let r = Resolution {
            tty: "/dev/ttys003".to_string(),
            cwd: "/x".to_string(),
            harness: Harness::Claude,
            session_id: Some("picked".to_string()),
            confidence: Confidence::Ambiguous,
            is_self: false,
            alternates,
        };
        let comment = alternates_comment(&r).unwrap();
        assert_eq!(comment.matches("id-").count(), MAX_ALTERNATES_SHOWN);
        assert!(comment.contains("+15 more"));
    }

    #[test]
    fn alternates_comment_is_none_for_non_ambiguous_or_empty_alternates() {
        let tab = Tab {
            tty: "/dev/ttys003".to_string(),
            cwd: "/x".to_string(),
        };
        let mut r = Resolution {
            tty: tab.tty.clone(),
            cwd: tab.cwd.clone(),
            harness: Harness::Claude,
            session_id: Some("only".to_string()),
            confidence: Confidence::Inferred,
            is_self: false,
            alternates: vec![(Harness::Codex, "alt".to_string())],
        };
        assert!(alternates_comment(&r).is_none()); // not ambiguous
        r.confidence = Confidence::Ambiguous;
        r.alternates.clear();
        assert!(alternates_comment(&r).is_none()); // no alternates
    }

    #[test]
    fn parse_args_defaults_to_in_place_and_parses_output_path() {
        let args = parse_args(&["-o".to_string(), "/tmp/x.sh".to_string()]).unwrap();
        assert_eq!(args.mode, Mode::InPlace);
        assert_eq!(args.out.as_deref(), Some("/tmp/x.sh"));

        let args = parse_args(&["--list".to_string()]).unwrap();
        assert_eq!(args.mode, Mode::List);

        assert!(parse_args(&["-o".to_string()]).is_err());
        assert!(parse_args(&["--bogus".to_string()]).is_err());
    }
}
