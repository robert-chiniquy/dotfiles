//! scorecard — render a dark, one-screen readiness scorecard TUI from a
//! structured markdown file. Truecolor ANSI, adapts to terminal width,
//! one line per criterion. No external dependencies.
//!
//! Usage:  scorecard [--width N] [--height N] [--mode fit|all] [--no-actions] <file.md>
//!         scorecard --action scorecard://remove/<id>?file=<path>
//!         scorecard install-handler | uninstall-handler        # macOS
//!         scorecard prime                                       # agent primer
//!
//! Content-groups: a line can belong to many groups. Built-in "type" groups are
//! auto-assigned — `header` (title/sub/meta), `titles` (section headers),
//! `callouts` (banners), `footer` — and topic groups are added per row with
//! `grp:<name>` cells (several allowed). Front matter `groups: name=priority, …`
//! sets priorities (default 0). Mode `fit` (default) drops whole groups
//! (lowest priority, then bottom-most) until the card fits in (height − 3) lines;
//! `all` renders everything. Removing any row in a group removes the group.

use std::collections::{HashMap, HashSet};
use std::env;
use std::process::exit;

// ---- palette ----
const ACCENT: [u8; 3] = [92, 236, 255];
const SPARK: [u8; 3] = [255, 0, 153];
const TEXT: [u8; 3] = [234, 232, 246];
const MUTED: [u8; 3] = [156, 151, 192];
const FAINT: [u8; 3] = [109, 104, 146];
const OK: [u8; 3] = [55, 224, 164];
const WARN: [u8; 3] = [251, 183, 37];
const CRITC: [u8; 3] = [255, 70, 111];

const RESET: &str = "\x1b[0m";
const BOLD: &str = "\x1b[1m";
const DIM: &str = "\x1b[2m";

fn c(rgb: [u8; 3]) -> String {
    format!("\x1b[38;2;{};{};{}m", rgb[0], rgb[1], rgb[2])
}
fn bg(rgb: [u8; 3]) -> String {
    format!("\x1b[48;2;{};{};{}m", rgb[0], rgb[1], rgb[2])
}
fn sty(rgb: [u8; 3], bold: bool, s: &str) -> String {
    format!("{}{}{}{}", c(rgb), if bold { BOLD } else { "" }, s, RESET)
}

// ---- visible-length-aware helpers (SGR + OSC 8 are zero-width) ----
fn vlen(s: &str) -> usize {
    let mut n = 0;
    let mut chars = s.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '\x1b' {
            match chars.peek() {
                Some('[') => {
                    chars.next();
                    while let Some(cc) = chars.next() {
                        if ('@'..='~').contains(&cc) {
                            break;
                        }
                    }
                }
                Some(']') => {
                    chars.next();
                    while let Some(cc) = chars.next() {
                        if cc == '\x07' {
                            break;
                        }
                        if cc == '\x1b' {
                            if chars.peek() == Some(&'\\') {
                                chars.next();
                            }
                            break;
                        }
                    }
                }
                _ => {
                    chars.next();
                }
            }
        } else {
            n += 1;
        }
    }
    n
}
fn pad_end(s: &str, w: usize) -> String {
    let v = vlen(s);
    if v >= w {
        s.to_string()
    } else {
        format!("{}{}", s, " ".repeat(w - v))
    }
}
fn trunc(s: &str, w: usize) -> String {
    let count = s.chars().count();
    if count <= w {
        return s.to_string();
    }
    if w == 0 {
        return String::new();
    }
    if w == 1 {
        return "…".to_string();
    }
    let head: String = s.chars().take(w - 1).collect();
    format!("{}…", head.trim_end())
}

// ---- percent-encoding for action URLs ----
fn percent_encode(s: &str) -> String {
    let mut out = String::new();
    for b in s.bytes() {
        let ch = b as char;
        if ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '.' | '~' | '/') {
            out.push(ch);
        } else {
            out.push_str(&format!("%{:02X}", b));
        }
    }
    out
}
fn percent_decode(s: &str) -> String {
    let bytes = s.as_bytes();
    let mut out = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let Ok(v) = u8::from_str_radix(&s[i + 1..i + 3], 16) {
                out.push(v);
                i += 3;
                continue;
            }
        }
        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

// ---- inline markdown links -> OSC 8 hyperlinks ----
#[derive(Debug, PartialEq)]
enum Seg {
    Text(String),
    Link { text: String, url: String },
}
fn find_char(chars: &[char], from: usize, target: char) -> Option<usize> {
    (from..chars.len()).find(|&j| chars[j] == target)
}
fn parse_links(s: &str) -> Vec<Seg> {
    let chars: Vec<char> = s.chars().collect();
    let mut segs = Vec::new();
    let mut buf = String::new();
    let mut i = 0;
    while i < chars.len() {
        if chars[i] == '[' {
            if let Some(close) = find_char(&chars, i + 1, ']') {
                if close + 1 < chars.len() && chars[close + 1] == '(' {
                    if let Some(paren) = find_char(&chars, close + 2, ')') {
                        let text: String = chars[i + 1..close].iter().collect();
                        let url: String = chars[close + 2..paren].iter().collect();
                        if !text.is_empty() && !url.is_empty() {
                            if !buf.is_empty() {
                                segs.push(Seg::Text(std::mem::take(&mut buf)));
                            }
                            segs.push(Seg::Link { text, url });
                            i = paren + 1;
                            continue;
                        }
                    }
                }
            }
        }
        buf.push(chars[i]);
        i += 1;
    }
    if !buf.is_empty() {
        segs.push(Seg::Text(buf));
    }
    segs
}
fn seg_vlen(segs: &[Seg]) -> usize {
    segs.iter()
        .map(|s| match s {
            Seg::Text(t) => t.chars().count(),
            Seg::Link { text, .. } => text.chars().count(),
        })
        .sum()
}
fn seg_trunc(segs: Vec<Seg>, w: usize) -> Vec<Seg> {
    if seg_vlen(&segs) <= w {
        return segs;
    }
    if w == 0 {
        return Vec::new();
    }
    let mut out = Vec::new();
    let mut budget = w;
    for s in segs {
        if budget == 0 {
            break;
        }
        match s {
            Seg::Text(t) => {
                let len = t.chars().count();
                if len <= budget {
                    budget -= len;
                    out.push(Seg::Text(t));
                } else {
                    let cut: String = t.chars().take(budget - 1).collect();
                    out.push(Seg::Text(format!("{}…", cut.trim_end())));
                    budget = 0;
                }
            }
            Seg::Link { text, url } => {
                let len = text.chars().count();
                if len <= budget {
                    budget -= len;
                    out.push(Seg::Link { text, url });
                } else {
                    let cut: String = text.chars().take(budget - 1).collect();
                    out.push(Seg::Link {
                        text: format!("{}…", cut.trim_end()),
                        url,
                    });
                    budget = 0;
                }
            }
        }
    }
    out
}
fn osc8(url: &str, text: &str) -> String {
    format!("\x1b]8;;{}\x1b\\{}\x1b]8;;\x1b\\", url, text)
}
fn render_field(text: &str, budget: usize, base: &str) -> String {
    let segs = seg_trunc(parse_links(text), budget);
    let mut out = String::from(base);
    for s in &segs {
        match s {
            Seg::Text(t) => out.push_str(t),
            Seg::Link { text, url } => out.push_str(&osc8(url, text)),
        }
    }
    out.push_str(RESET);
    out
}

// ---- model ----
#[derive(Clone, Copy, PartialEq, Debug)]
enum Sev {
    Ok,
    Warn,
    Crit,
}
impl Sev {
    fn from(s: &str) -> Sev {
        match s.trim().to_lowercase().as_str() {
            "ok" | "solid" | "green" | "done" | "good" => Sev::Ok,
            "gap" | "crit" | "critical" | "red" | "block" | "blocked" | "fail" => Sev::Crit,
            _ => Sev::Warn,
        }
    }
    fn color(self) -> [u8; 3] {
        match self {
            Sev::Ok => OK,
            Sev::Warn => WARN,
            Sev::Crit => CRITC,
        }
    }
    fn bgc(self) -> [u8; 3] {
        match self {
            Sev::Ok => [17, 46, 38],
            Sev::Warn => [46, 38, 12],
            Sev::Crit => [46, 16, 24],
        }
    }
    fn label(self) -> &'static str {
        match self {
            Sev::Ok => "SOLID",
            Sev::Warn => "AT RISK",
            Sev::Crit => "GAP",
        }
    }
}

struct Crit {
    id: String,
    sev: Sev,
    score: String,
    name: String,
    note: String,
    groups: Vec<String>,
}
struct Banner {
    tag: String,
    text: String,
    groups: Vec<String>,
}
enum Kind {
    Crit,
    Banner,
}
struct Section {
    label: String,
    weight: String,
    kind: Kind,
    crits: Vec<Crit>,
    banners: Vec<Banner>,
}
#[derive(Default)]
struct Doc {
    title: String,
    sub: String,
    meta: String,
    note: String,
    footer: String,
    score: Option<(u32, u32)>,
    pass: Option<u32>,
    groups: HashMap<String, i32>,
    sections: Vec<Section>,
}

// membership: which group names a rendered element belongs to.
fn crit_membership(si: usize, cr: &Crit) -> Vec<String> {
    if cr.groups.is_empty() {
        vec![format!("~{}:{}", si, cr.id)] // singleton, droppable on its own
    } else {
        cr.groups.clone()
    }
}
fn banner_membership(b: &Banner) -> Vec<String> {
    let mut v = vec!["callouts".to_string()];
    v.extend(b.groups.iter().cloned());
    v
}
fn crit_visible(si: usize, cr: &Crit, hidden: &HashSet<String>) -> bool {
    crit_membership(si, cr).iter().all(|g| !hidden.contains(g))
}
fn banner_visible(b: &Banner, hidden: &HashSet<String>) -> bool {
    banner_membership(b).iter().all(|g| !hidden.contains(g))
}

// ---- parsing ----
fn split_row(line: &str) -> Vec<String> {
    let mut parts: Vec<String> = line.split('|').map(|c| c.trim().to_string()).collect();
    if parts.first().map_or(false, |s| s.is_empty()) {
        parts.remove(0);
    }
    if parts.last().map_or(false, |s| s.is_empty()) {
        parts.pop();
    }
    parts
}
fn is_sep(cells: &[String]) -> bool {
    !cells.is_empty()
        && cells.iter().all(|c| {
            let c = c.trim();
            !c.is_empty() && c.chars().all(|ch| ch == '-' || ch == ':')
        })
}
fn is_header(cells: &[String]) -> bool {
    matches!(
        cells.first().map(|s| s.to_lowercase()).as_deref(),
        Some("id" | "tag" | "#")
    )
}
fn split_weight(h: &str) -> (String, String) {
    if let Some(i) = h.find('(') {
        (
            h[..i].trim().to_string(),
            h[i + 1..].trim_end_matches(')').trim().to_string(),
        )
    } else {
        (h.trim().to_string(), String::new())
    }
}
fn parse_score(v: &str) -> Option<(u32, u32)> {
    let (a, b) = v.split_once('/')?;
    Some((a.trim().parse().ok()?, b.trim().parse().ok()?))
}
fn is_banner_label(label: &str) -> bool {
    matches!(
        label.trim().to_lowercase().as_str(),
        "callouts" | "callout" | "banners" | "banner" | "notes" | "note"
    )
}
fn cell_group(cell: &str) -> Option<String> {
    cell.strip_prefix("grp:")
        .or_else(|| cell.strip_prefix("group:"))
        .map(|g| g.trim().to_string())
}
/// Pull every `grp:<name>` cell out, returning the group names and remaining cells.
fn extract_groups(cells: &[String]) -> (Vec<String>, Vec<String>) {
    let mut groups = Vec::new();
    let mut rest = Vec::new();
    for cell in cells {
        match cell_group(cell) {
            Some(g) => groups.push(g),
            None => rest.push(cell.clone()),
        }
    }
    (groups, rest)
}
fn row_groups(cells: &[String]) -> Vec<String> {
    cells.iter().filter_map(|c| cell_group(c)).collect()
}
fn parse(input: &str) -> Doc {
    let mut doc = Doc::default();
    let mut cur: Option<Section> = None;
    let mut in_front = true;
    for raw in input.lines() {
        let t = raw.trim();
        if t.is_empty() {
            continue;
        }
        if let Some(h2) = t.strip_prefix("## ") {
            if let Some(s) = cur.take() {
                doc.sections.push(s);
            }
            in_front = false;
            let (label, weight) = split_weight(h2.trim());
            let kind = if is_banner_label(&label) {
                Kind::Banner
            } else {
                Kind::Crit
            };
            cur = Some(Section {
                label,
                weight,
                kind,
                crits: Vec::new(),
                banners: Vec::new(),
            });
            continue;
        }
        if let Some(h1) = t.strip_prefix("# ") {
            if doc.title.is_empty() {
                doc.title = h1.trim().to_string();
            }
            continue;
        }
        if t.starts_with('|') {
            let cells = split_row(t);
            if cells.is_empty() || is_sep(&cells) || is_header(&cells) {
                continue;
            }
            if let Some(s) = cur.as_mut() {
                match s.kind {
                    Kind::Banner => {
                        if cells.len() >= 2 {
                            let (groups, rest) = extract_groups(&cells[1..]);
                            s.banners.push(Banner {
                                tag: cells[0].clone(),
                                text: rest.join(" ").trim().to_string(),
                                groups,
                            });
                        }
                    }
                    Kind::Crit => {
                        if cells.len() >= 4 {
                            let (groups, rest) = extract_groups(&cells[4..]);
                            s.crits.push(Crit {
                                id: cells[0].clone(),
                                sev: Sev::from(&cells[1]),
                                score: cells[2].clone(),
                                name: cells[3].clone(),
                                note: rest.join(" | ").trim().to_string(),
                                groups,
                            });
                        }
                    }
                }
            }
            continue;
        }
        if t.starts_with("> ") {
            if doc.footer.is_empty() {
                doc.footer = t[2..].trim().to_string();
            }
            continue;
        }
        if in_front {
            if let Some((k, v)) = t.split_once(':') {
                let key = k.trim().to_lowercase();
                let val = v.trim().to_string();
                match key.as_str() {
                    "title" => doc.title = val,
                    "sub" | "subtitle" => doc.sub = val,
                    "meta" => doc.meta = val,
                    "note" => doc.note = val,
                    "footer" => doc.footer = val,
                    "score" => doc.score = parse_score(&val),
                    "pass" | "threshold" => doc.pass = val.parse().ok(),
                    "groups" => {
                        for part in val.split(',') {
                            if let Some((n, p)) = part.split_once('=') {
                                if let Ok(pr) = p.trim().parse::<i32>() {
                                    doc.groups.insert(n.trim().to_string(), pr);
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
    }
    if let Some(s) = cur.take() {
        doc.sections.push(s);
    }
    doc
}

// ---- actions ----
/// Remove the row with `id`; if it carries topic groups, remove every row that
/// shares any of them. Returns the number of rows removed.
fn remove_line(file: &str, id: &str) -> Result<usize, String> {
    if !file.ends_with(".md") {
        return Err(format!("refusing to edit non-markdown file: {}", file));
    }
    let content = std::fs::read_to_string(file).map_err(|e| format!("read {}: {}", file, e))?;

    let mut target: Vec<String> = Vec::new();
    for line in content.lines() {
        let t = line.trim_start();
        if t.starts_with('|') {
            let cells = split_row(t);
            if cells.first().map(|s| s.as_str()) == Some(id) {
                target = row_groups(&cells);
                break;
            }
        }
    }

    let mut kept: Vec<&str> = Vec::new();
    let mut removed = 0;
    for line in content.lines() {
        let t = line.trim_start();
        if t.starts_with('|') {
            let cells = split_row(t);
            let drop = if target.is_empty() {
                cells.first().map(|s| s.as_str()) == Some(id)
            } else {
                row_groups(&cells).iter().any(|g| target.contains(g))
            };
            if drop {
                removed += 1;
                continue;
            }
        }
        kept.push(line);
    }
    if removed > 0 {
        let mut new = kept.join("\n");
        new.push('\n');
        std::fs::write(file, new).map_err(|e| format!("write {}: {}", file, e))?;
    }
    Ok(removed)
}
fn run_action(url: &str) -> Result<String, String> {
    let rest = url
        .strip_prefix("scorecard://")
        .ok_or_else(|| format!("not a scorecard URL: {}", url))?;
    let (pathq, query) = rest.split_once('?').unwrap_or((rest, ""));
    let segs: Vec<&str> = pathq.split('/').filter(|s| !s.is_empty()).collect();
    let action = segs.first().copied().unwrap_or("");
    let mut file = String::new();
    for kv in query.split('&') {
        if let Some(v) = kv.strip_prefix("file=") {
            file = percent_decode(v);
        }
    }
    match action {
        "remove" => {
            let id = segs.get(1).copied().unwrap_or("");
            if id.is_empty() {
                return Err("remove: missing id".into());
            }
            if file.is_empty() {
                return Err("remove: missing file".into());
            }
            let n = remove_line(&file, id)?;
            Ok(format!("removed {} ({} line{})", id, n, if n == 1 { "" } else { "s" }))
        }
        other => Err(format!("unknown action: {}", other)),
    }
}
/// List the current line items in `file` as markdown, for agents to diff
/// against what they wrote (the user may have removed some via the ✕ close box).
fn list_items(file: &str) -> Result<String, String> {
    let content = std::fs::read_to_string(file).map_err(|e| format!("read {}: {}", file, e))?;
    let doc = parse(&content);
    let mut out = String::new();
    for s in &doc.sections {
        for cr in &s.crits {
            let groups = if cr.groups.is_empty() {
                String::new()
            } else {
                format!(" · groups: {}", cr.groups.join(", "))
            };
            out.push_str(&format!("- `{}` {} — {}{}\n", cr.id, cr.name, cr.sev.label(), groups));
        }
    }
    Ok(out)
}
fn notify(msg: &str) {
    let _ = std::process::Command::new("osascript")
        .arg("-e")
        .arg(format!("display notification {:?} with title \"scorecard\"", msg))
        .status();
}

// ---- self-install of the scorecard:// handler (macOS, Route B) ----
fn run_cmd(cmd: &mut std::process::Command) -> Result<(), String> {
    let st = cmd.status().map_err(|e| format!("spawn failed: {}", e))?;
    if st.success() {
        Ok(())
    } else {
        Err(format!("{:?} exited with {}", cmd, st))
    }
}
#[cfg(target_os = "macos")]
fn install_handler() -> Result<String, String> {
    use std::process::Command;
    let bin = std::env::current_exe()
        .map_err(|e| format!("current_exe: {}", e))?
        .to_string_lossy()
        .into_owned();
    let home = env::var("HOME").map_err(|_| "HOME unset".to_string())?;
    let app = format!("{}/Applications/ScorecardHandler.app", home);
    let tmp = env::temp_dir().join(format!("scorecard-handler-{}", std::process::id()));
    std::fs::create_dir_all(&tmp).map_err(|e| e.to_string())?;
    let script = tmp.join("handler.applescript");
    std::fs::write(
        &script,
        format!(
            "on open location this_URL\n\tdo shell script \"{} --action \" & quoted form of this_URL\nend open location\n",
            bin
        ),
    )
    .map_err(|e| e.to_string())?;
    let _ = std::fs::remove_dir_all(&app);
    std::fs::create_dir_all(format!("{}/Applications", home)).ok();
    run_cmd(Command::new("osacompile").arg("-o").arg(&app).arg(&script))?;
    let plist = format!("{}/Contents/Info.plist", app);
    let pb = "/usr/libexec/PlistBuddy";
    let _ = Command::new(pb).args(["-c", "Add :CFBundleURLTypes array", &plist]).status();
    run_cmd(Command::new(pb).args(["-c", "Add :CFBundleURLTypes:0 dict", &plist]))?;
    run_cmd(Command::new(pb).args(["-c", "Add :CFBundleURLTypes:0:CFBundleURLName string ai.c1.scorecard", &plist]))?;
    run_cmd(Command::new(pb).args(["-c", "Add :CFBundleURLTypes:0:CFBundleURLSchemes array", &plist]))?;
    run_cmd(Command::new(pb).args(["-c", "Add :CFBundleURLTypes:0:CFBundleURLSchemes:0 string scorecard", &plist]))?;
    let lsregister = "/System/Library/Frameworks/CoreServices.framework/Frameworks/LaunchServices.framework/Support/lsregister";
    run_cmd(Command::new(lsregister).arg("-f").arg(&app))?;
    let _ = std::fs::remove_dir_all(&tmp);
    Ok(format!("registered scorecard:// -> {}\n  handler: {}", bin, app))
}
#[cfg(target_os = "macos")]
fn uninstall_handler() -> Result<String, String> {
    let home = env::var("HOME").map_err(|_| "HOME unset".to_string())?;
    let app = format!("{}/Applications/ScorecardHandler.app", home);
    let lsregister = "/System/Library/Frameworks/CoreServices.framework/Frameworks/LaunchServices.framework/Support/lsregister";
    let _ = std::process::Command::new(lsregister).arg("-u").arg(&app).status();
    std::fs::remove_dir_all(&app).map_err(|e| format!("remove {}: {}", app, e))?;
    Ok(format!("removed {}", app))
}
#[cfg(not(target_os = "macos"))]
fn install_handler() -> Result<String, String> {
    Err("install-handler is macOS-only".into())
}
#[cfg(not(target_os = "macos"))]
fn uninstall_handler() -> Result<String, String> {
    Err("uninstall-handler is macOS-only".into())
}

// ---- install-agents: point every installed harness at `scorecard prime` ----
const AGENTS_BEGIN: &str = "<!-- scorecard:begin (managed by `scorecard install-agents`) -->";
const AGENTS_END: &str = "<!-- scorecard:end -->";
fn agents_block() -> String {
    format!(
        "{}\nWhen asked to summarize a tactical situation — a code / PR / incident / deadline /\nmilestone status, a readiness read, or a go/no-go — run `scorecard prime` and follow it\nto render a skimmable status scorecard with the `scorecard` TUI.\n{}",
        AGENTS_BEGIN, AGENTS_END
    )
}
fn upsert_block(content: &str, block: &str) -> String {
    if let (Some(bi), Some(ei)) = (content.find(AGENTS_BEGIN), content.find(AGENTS_END)) {
        let end = ei + AGENTS_END.len();
        format!("{}{}{}", &content[..bi], block, &content[end..])
    } else if content.trim().is_empty() {
        format!("{}\n", block)
    } else {
        format!("{}\n\n{}\n", content.trim_end(), block)
    }
}
fn remove_block(content: &str) -> String {
    if let (Some(bi), Some(ei)) = (content.find(AGENTS_BEGIN), content.find(AGENTS_END)) {
        let end = ei + AGENTS_END.len();
        let joined = format!("{}{}", content[..bi].trim_end(), &content[end..]);
        format!("{}\n", joined.trim())
    } else {
        content.to_string()
    }
}
// (display name, detect-dir under $HOME, instructions file under $HOME)
const HARNESSES: &[(&str, &str, &str)] = &[
    ("Claude Code", ".claude", ".claude/CLAUDE.md"),
    ("Codex", ".codex", ".codex/AGENTS.md"),
    ("Cursor", ".cursor", ".cursor/rules/scorecard.mdc"),
    ("pi", ".pi", ".pi/AGENTS.md"),
    ("opencode", ".opencode", ".opencode/AGENTS.md"),
    ("Goose", ".config/goose", ".config/goose/AGENTS.md"),
];
fn agents_apply(remove: bool) -> Result<String, String> {
    let home = env::var("HOME").map_err(|_| "HOME unset".to_string())?;
    let block = agents_block();
    let mut report = Vec::new();
    for (name, dir, file) in HARNESSES {
        let dirp = format!("{}/{}", home, dir);
        if !std::path::Path::new(&dirp).exists() {
            report.push(format!("  skip   {:<12} (not installed)", name));
            continue;
        }
        let filep = format!("{}/{}", home, file);
        if let Some(parent) = std::path::Path::new(&filep).parent() {
            std::fs::create_dir_all(parent).ok();
        }
        let existing = std::fs::read_to_string(&filep).unwrap_or_default();
        let updated = if remove {
            remove_block(&existing)
        } else {
            upsert_block(&existing, &block)
        };
        std::fs::write(&filep, updated).map_err(|e| format!("write {}: {}", filep, e))?;
        report.push(format!("  {}  {:<12} {}", if remove { "clear" } else { "wrote" }, name, file));
    }
    Ok(report.join("\n"))
}

// ---- rendering ----
struct Canvas {
    w: usize,
    iw: usize,
    nw: usize,
    lines: Vec<String>,
}
impl Canvas {
    fn new(w: usize) -> Canvas {
        let iw = w - 4;
        let nw = ((iw as f64 * 0.30).round() as usize).clamp(24, 40);
        Canvas {
            w,
            iw,
            nw,
            lines: Vec::new(),
        }
    }
    fn raw(&mut self, s: String) {
        self.lines.push(s);
    }
    fn boxln(&mut self, content: &str) {
        self.lines.push(format!(
            "{}│{} {} {}│{}",
            c(FAINT),
            RESET,
            pad_end(content, self.iw),
            c(FAINT),
            RESET
        ));
    }
    fn top(&mut self) {
        self.raw(format!("{}┌{}┐{}", c(FAINT), "─".repeat(self.w - 2), RESET));
    }
    fn bot(&mut self) {
        self.raw(format!("{}└{}┘{}", c(FAINT), "─".repeat(self.w - 2), RESET));
    }
    fn rule(&mut self) {
        self.raw(format!("{}├{}┤{}", c(FAINT), "─".repeat(self.w - 2), RESET));
    }
}
fn pill(sev: Sev) -> String {
    let label = sev.label();
    let chip = format!("{}{}{} {} {}", bg(sev.bgc()), c(sev.color()), BOLD, label, RESET);
    let pad = 9usize.saturating_sub(label.len() + 2);
    format!("{}{}", chip, " ".repeat(pad))
}
fn banner_color(tag: &str) -> [u8; 3] {
    let u = tag.to_uppercase();
    if u.contains("STANDOUT") || u.contains("RISK") {
        SPARK
    } else if u.contains("DECIDE") || u.contains("BLOCK") || u.contains("GAP") {
        CRITC
    } else {
        ACCENT
    }
}

fn crit_column_widths(doc: &Doc, hidden: &HashSet<String>, id_budget: usize) -> (usize, usize) {
    let mut id_width = 1;
    let mut score_width = 1;
    for (si, section) in doc.sections.iter().enumerate() {
        for cr in &section.crits {
            if !crit_visible(si, cr, hidden) {
                continue;
            }
            id_width = id_width.max(seg_vlen(&parse_links(&cr.id)).min(id_budget));
            score_width = score_width.max(cr.score.chars().count());
        }
    }
    (id_width, score_width)
}

fn build_lines(doc: &Doc, w: usize, file: Option<&str>, hidden: &HashSet<String>) -> Vec<String> {
    let mut cv = Canvas::new(w);
    let iw = cv.iw;
    let (id_width, score_width) = crit_column_widths(doc, hidden, cv.nw);

    cv.top();
    if !hidden.contains("header") {
        if !doc.title.is_empty() {
            cv.boxln(&sty(ACCENT, true, &trunc(&doc.title, iw)));
        }
        if !doc.sub.is_empty() {
            cv.boxln(&sty(MUTED, false, &trunc(&doc.sub, iw)));
        }
        if !doc.meta.is_empty() {
            cv.boxln(&sty(FAINT, false, &trunc(&doc.meta, iw)));
        }
    }
    cv.rule();

    let mut show_meter = false;
    if let Some((proj, max)) = doc.score {
        if max > 0 && !hidden.contains("meter") {
            show_meter = true;
            let pass = doc.pass.unwrap_or(((max as f64) * 0.7).round() as u32);
            let cells = iw.saturating_sub(34).max(10);
            let fill = (((proj as f64) / (max as f64)) * cells as f64).round() as usize;
            let thr = (((pass as f64) / (max as f64)) * cells as f64).round() as usize;
            let mut bar = String::new();
            for i in 0..cells {
                if i == thr {
                    bar.push_str(&format!("{}{}│{}", c(TEXT), BOLD, RESET));
                } else if i < fill {
                    let p = if fill > 1 {
                        i as f64 / (fill - 1) as f64
                    } else {
                        0.0
                    };
                    let col = [0, 1, 2]
                        .map(|k| (ACCENT[k] as f64 + (SPARK[k] as f64 - ACCENT[k] as f64) * p).round() as u8);
                    bar.push_str(&format!("{}█{}", c(col), RESET));
                } else {
                    bar.push_str(&format!("{}·{}", c(FAINT), RESET));
                }
            }
            let pct = ((proj as f64 / max as f64) * 100.0).round() as u32;
            cv.boxln(&format!(
                "{}PROJECTED {}{}~{}{}{}/{} {}~{}%{}  {}  {}{}│{}{}",
                c(FAINT), c(TEXT), BOLD, proj, RESET, c(FAINT), max, c(WARN), pct, RESET, bar, c(TEXT), BOLD, c(FAINT), pass
            ));
        }
    }
    let (mut n_ok, mut n_warn, mut n_crit, mut n_zero) = (0, 0, 0, 0);
    for (si, s) in doc.sections.iter().enumerate() {
        for cr in &s.crits {
            if !crit_visible(si, cr, hidden) {
                continue;
            }
            match cr.sev {
                Sev::Ok => n_ok += 1,
                Sev::Warn => n_warn += 1,
                Sev::Crit => n_crit += 1,
            }
            if cr.score.trim() == "0" {
                n_zero += 1;
            }
        }
    }
    let mut show_tiles = false;
    if n_ok + n_warn + n_crit > 0 && !hidden.contains("tiles") {
        show_tiles = true;
        let mut tiles = String::new();
        tiles += &sty(OK, true, &n_ok.to_string());
        tiles += &sty(MUTED, false, " solid   ");
        tiles += &sty(WARN, true, &n_warn.to_string());
        tiles += &sty(MUTED, false, " at-risk   ");
        tiles += &sty(CRITC, true, &n_crit.to_string());
        tiles += &sty(MUTED, false, " gap   ");
        tiles += &sty(TEXT, true, &n_zero.to_string());
        tiles += &sty(MUTED, false, " at zero");
        if !doc.note.is_empty() {
            let budget = iw.saturating_sub(vlen(&tiles) + 4);
            if budget > 6 {
                tiles += &format!("   {}", render_field(&doc.note, budget, &format!("{}{}", DIM, c(MUTED))));
            }
        }
        cv.boxln(&tiles);
    }
    if show_meter || show_tiles {
        cv.rule();
    }

    let titles_hidden = hidden.contains("titles");
    let mut banner_rule_done = false;
    for (si, s) in doc.sections.iter().enumerate() {
        match s.kind {
            Kind::Crit => {
                let vis: Vec<&Crit> = s.crits.iter().filter(|cr| crit_visible(si, cr, hidden)).collect();
                if vis.is_empty() {
                    continue;
                }
                if !titles_hidden {
                    let hdr = if s.weight.is_empty() {
                        sty(TEXT, true, &s.label)
                    } else {
                        format!("{} {}", sty(TEXT, true, &s.label), sty(FAINT, false, &s.weight))
                    };
                    cv.boxln(&hdr);
                }
                for cr in vis {
                    let close = match file {
                        Some(f) => {
                            let url = format!("scorecard://remove/{}?file={}", cr.id, percent_encode(f));
                            format!(" {}", osc8(&url, &format!("{}✕{}", c(FAINT), RESET)))
                        }
                        None => String::new(),
                    };
                    let cwid = vlen(&close);
                    let id_r = pad_end(
                        &render_field(&cr.id, id_width, &format!("{}{}", c(cr.sev.color()), BOLD)),
                        id_width,
                    );
                    let name = pad_end(&trunc(&cr.name, cv.nw), cv.nw);
                    let score = pad_end(&cr.score, score_width);
                    let left = format!(
                        "{}▌{} {} {}{} {} {}{}{}{}",
                        c(cr.sev.color()), RESET, id_r, c(TEXT), name, pill(cr.sev), c(cr.sev.color()), BOLD, score, RESET
                    );
                    let budget = iw.saturating_sub(vlen(&left) + 1 + cwid);
                    let note = if budget > 6 && !cr.note.is_empty() {
                        format!(" {}", render_field(&cr.note, budget, &format!("{}{}", DIM, c(MUTED))))
                    } else {
                        String::new()
                    };
                    let body = format!("{}{}", left, note);
                    cv.boxln(&format!("{}{}", pad_end(&body, iw.saturating_sub(cwid)), close));
                }
            }
            Kind::Banner => {
                let vis: Vec<&Banner> = s.banners.iter().filter(|b| banner_visible(b, hidden)).collect();
                if vis.is_empty() {
                    continue;
                }
                if !banner_rule_done {
                    cv.rule();
                    banner_rule_done = true;
                }
                for b in vis {
                    let head = sty(banner_color(&b.tag), true, &pad_end(&b.tag, 9));
                    let budget = iw.saturating_sub(vlen(&head));
                    cv.boxln(&format!("{}{}", head, render_field(&b.text, budget, &c(MUTED))));
                }
            }
        }
    }

    cv.rule();
    if !doc.footer.is_empty() && !hidden.contains("footer") {
        cv.boxln(&format!("{}{}{}{}", DIM, c(FAINT), trunc(&doc.footer, iw), RESET));
    }
    cv.bot();
    cv.lines
}
fn render(doc: &Doc, w: usize, file: Option<&str>, hidden: &HashSet<String>) -> String {
    format!("\n{}\n", build_lines(doc, w, file, hidden).join("\n"))
}

// ---- fit: hide whole content-groups (lowest priority first) until it fits ----
fn bump(m: &mut HashMap<String, usize>, name: &str, pos: usize) {
    let e = m.entry(name.to_string()).or_insert(0);
    if pos > *e {
        *e = pos;
    }
}
// Structural "chrome" groups yield to line items: chrome defaults low, so fit
// sheds it (header/titles/meter/tiles/callouts/footer) before dropping any item.
fn is_chrome(name: &str) -> bool {
    matches!(name, "header" | "titles" | "meter" | "tiles" | "callouts" | "footer")
}
fn default_priority(name: &str) -> i32 {
    if is_chrome(name) {
        0
    } else {
        10 // line items (singletons + topic groups) outrank chrome
    }
}
/// Every group present in the doc: (name, priority, bottom-most position).
fn group_universe(doc: &Doc) -> Vec<(String, i32, usize)> {
    let mut posmap: HashMap<String, usize> = HashMap::new();
    let mut pos = 0usize;
    if !doc.title.is_empty() || !doc.sub.is_empty() || !doc.meta.is_empty() {
        pos += 1;
        bump(&mut posmap, "header", pos);
    }
    if doc.score.is_some() {
        pos += 1;
        bump(&mut posmap, "meter", pos);
    }
    if doc.sections.iter().any(|s| !s.crits.is_empty()) {
        pos += 1;
        bump(&mut posmap, "tiles", pos);
    }
    for (si, s) in doc.sections.iter().enumerate() {
        pos += 1;
        bump(&mut posmap, "titles", pos);
        for cr in &s.crits {
            pos += 1;
            for g in crit_membership(si, cr) {
                bump(&mut posmap, &g, pos);
            }
        }
        for b in &s.banners {
            pos += 1;
            for g in banner_membership(b) {
                bump(&mut posmap, &g, pos);
            }
        }
    }
    if !doc.footer.is_empty() {
        pos += 1;
        bump(&mut posmap, "footer", pos);
    }
    posmap
        .into_iter()
        .map(|(name, p)| {
            let prio = doc.groups.get(&name).copied().unwrap_or_else(|| default_priority(&name));
            (name, prio, p)
        })
        .collect()
}
/// Choose the group names to hide so the card fits in `avail` rows.
fn fit(doc: &Doc, w: usize, file: Option<&str>, avail: usize) -> HashSet<String> {
    let mut hidden = HashSet::new();
    loop {
        if build_lines(doc, w, file, &hidden).len() <= avail {
            break;
        }
        let pick = group_universe(doc)
            .into_iter()
            .filter(|(n, _, _)| !hidden.contains(n))
            .min_by(|a, b| a.1.cmp(&b.1).then_with(|| b.2.cmp(&a.2)));
        match pick {
            Some((n, _, _)) => {
                hidden.insert(n);
            }
            None => break,
        }
    }
    hidden
}

fn term_width(cli: Option<usize>) -> usize {
    cli.or_else(|| env::var("COLUMNS").ok().and_then(|s| s.trim().parse::<usize>().ok()))
        .unwrap_or(120)
        .clamp(84, 170)
}
fn term_height(cli: Option<usize>) -> Option<usize> {
    cli.or_else(|| env::var("LINES").ok().and_then(|s| s.trim().parse::<usize>().ok()))
}

#[derive(PartialEq)]
enum Mode {
    Fit,
    All,
}

const PRIME: &str = r#"# scorecard — agent primer

A dark, one-screen readiness/status TUI rendered in the terminal from a
structured markdown file. You (an agent) write the file; `scorecard <file>`
renders it. Not a web page.

## The recipe you usually want

Write ~/.config/scorecard/status.md (preserve the old one first — see below),
then it shows on the user's next terminal window (a shell greeting renders it).

    # Title
    sub: one-line subtitle
    meta: deadline · pass bar · weights
    score: 155/215                 # optional; drives the meter
    note: short line shown by the tiles
    footer: dim footer line
    groups: header=100, callouts=5, tiers=8    # group -> priority (higher = kept)

    ## Theme (x3)
    | id | state | score | criterion | note |
    | D1 | risk  | 3 | Ship the build | rides [T-1](https://linear.app/…) | grp:tiers |

    ## Callouts
    | STANDOUT | the single most important thing |
    | NEXT | the next actions |

state = solid | risk | gap  (green / gold / red). A row may carry several
`grp:<name>` cells. Notes/callouts accept `[text](url)` -> clickable links.

## Content-groups

A line can belong to many groups. Built-in "type" groups are automatic:
`header` (title/sub/meta), `titles` (section headers), `callouts` (banners),
`footer`. Topic groups are the `grp:<name>` tags. Set priorities in the
front-matter `groups:` line (default 0; negatives drop first).

## Modes

- fit (default): drop whole groups — lowest priority, then bottom-most — until
  the card fits in (terminal height − 3) lines. By default chrome (header,
  titles, meter, tiles, callouts, footer) is shed before any line item, so the
  list of items survives longest.
- all: render everything.   Flag: --mode all|fit
Pass --width "$COLUMNS" --height "$LINES" from a prompt hook.

## Actions

Each row shows a ✕ close box linking to scorecard://remove/<id>. Register the
scheme once (macOS): `scorecard install-handler` (uninstall-handler to remove).
Clicking removes that row — and every row sharing its topic group — from the file.

## It's live — re-check what remains

The scorecard is shared state: the user removes items by clicking a row's ✕
(which deletes that row and any rows sharing its group). If you are tracking work
you wrote here, do not assume your items survived — periodically re-read what
remains. `scorecard list [file]` prints the current items as markdown (id, name,
state); diff it against what you wrote to see what the user dropped.

And prune proactively: if you notice an item has gone stale — a branch that
merged, a ticket that closed, work that finished — just remove it, no need to
ask. Drop the row from the file, or run
`scorecard --action scorecard://remove/<id>?file=<path>`.

## Compose well

- Scope to the user's surface: only what they own, drive, are blocked on, or must
  decide — drop work that sits elsewhere.
- Group by theme (the clusters they actually think in), not generic Must/Should
  tiers unless the rubric is truly weighted (e.g. a scored POC).
- Succinct, weighted by importance: trim each note; let a line's length track how
  much it matters — the one real gap earns its whole line, routine solids get a
  few words. Uniform-length notes flatten the signal.
- Honest severities; real content only; no names or PII in anything shareable.
- Anchor each item to a larger concept: when the line has horizontal room, name
  the ticket theme, repo, or system it belongs to (`[IGA-3293]`, `pqprime`,
  `MLS`) so it skims fast.
- No past tense: DONE / COMPLETED / COMMITTED / MERGED items do not belong — the
  scorecard tracks what is live and ahead, not a changelog. (A `solid` *state* is
  fine — a live thing going well; a purely finished item is clutter, drop it.)

## Preserve-on-write

Before writing a new status.md, move the old one aside with a datestamp:
    mv ~/.config/scorecard/status.md ~/.config/scorecard/status-$(date +%F).md
"#;

fn main() {
    let argv: Vec<String> = env::args().skip(1).collect();
    match argv.first().map(|s| s.as_str()) {
        Some("list") => {
            let file = argv.get(1).cloned().unwrap_or_else(|| {
                env::var("SCORECARD_FILE").unwrap_or_else(|_| {
                    format!("{}/.config/scorecard/status.md", env::var("HOME").unwrap_or_default())
                })
            });
            match list_items(&file) {
                Ok(md) => {
                    print!("{}", md);
                    exit(0);
                }
                Err(e) => {
                    eprintln!("scorecard: {}", e);
                    exit(1);
                }
            }
        }
        Some("prime") => {
            print!("{}", PRIME);
            exit(0);
        }
        Some("install-agents") => match agents_apply(false) {
            Ok(m) => {
                println!("scorecard: pointed installed harnesses at `scorecard prime`\n{}", m);
                exit(0);
            }
            Err(e) => {
                eprintln!("scorecard: {}", e);
                exit(1);
            }
        },
        Some("uninstall-agents") => match agents_apply(true) {
            Ok(m) => {
                println!("scorecard: removed the scorecard block from harnesses\n{}", m);
                exit(0);
            }
            Err(e) => {
                eprintln!("scorecard: {}", e);
                exit(1);
            }
        },
        Some("install-handler") => match install_handler() {
            Ok(m) => {
                println!("{}", m);
                exit(0);
            }
            Err(e) => {
                eprintln!("scorecard: {}", e);
                exit(1);
            }
        },
        Some("uninstall-handler") => match uninstall_handler() {
            Ok(m) => {
                println!("{}", m);
                exit(0);
            }
            Err(e) => {
                eprintln!("scorecard: {}", e);
                exit(1);
            }
        },
        _ => {}
    }

    let mut width_cli: Option<usize> = None;
    let mut height_cli: Option<usize> = None;
    let mut mode = Mode::Fit;
    let mut file: Option<String> = None;
    let mut action: Option<String> = None;
    let mut no_actions = false;
    let mut args = argv.into_iter();
    while let Some(a) = args.next() {
        if a == "--width" {
            width_cli = args.next().and_then(|s| s.trim().parse().ok());
        } else if let Some(v) = a.strip_prefix("--width=") {
            width_cli = v.trim().parse().ok();
        } else if a == "--height" {
            height_cli = args.next().and_then(|s| s.trim().parse().ok());
        } else if let Some(v) = a.strip_prefix("--height=") {
            height_cli = v.trim().parse().ok();
        } else if a == "--mode" {
            if let Some(m) = args.next() {
                mode = if m == "all" { Mode::All } else { Mode::Fit };
            }
        } else if let Some(v) = a.strip_prefix("--mode=") {
            mode = if v == "all" { Mode::All } else { Mode::Fit };
        } else if a == "--action" {
            action = args.next();
        } else if a == "--no-actions" {
            no_actions = true;
        } else if a == "-h" || a == "--help" {
            eprintln!("usage: scorecard [--width N] [--height N] [--mode fit|all] [--no-actions] <file.md>");
            eprintln!("       scorecard --action scorecard://remove/<id>?file=<path>");
            eprintln!("       scorecard install-handler | uninstall-handler | prime");
            eprintln!("       scorecard install-agents | uninstall-agents");
            eprintln!("       scorecard list [file.md]");
            exit(0);
        } else if a.starts_with("scorecard://") {
            action = Some(a);
        } else if !a.starts_with('-') {
            file = Some(a);
        }
    }

    if let Some(url) = action {
        match run_action(&url) {
            Ok(msg) => {
                notify(&msg);
                exit(0);
            }
            Err(e) => {
                eprintln!("scorecard: {}", e);
                exit(1);
            }
        }
    }

    let path = match file {
        Some(p) => p,
        None => {
            eprintln!("usage: scorecard [--width N] [--height N] [--mode fit|all] [--no-actions] <file.md>");
            exit(2);
        }
    };
    let input = match std::fs::read_to_string(&path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("scorecard: cannot read {}: {}", path, e);
            exit(1);
        }
    };
    let abs = std::fs::canonicalize(&path)
        .map(|p| p.to_string_lossy().into_owned())
        .unwrap_or_else(|_| path.clone());
    let w = term_width(width_cli);
    let file_ref = if no_actions { None } else { Some(abs.as_str()) };
    let doc = parse(&input);
    let hidden = match (&mode, term_height(height_cli)) {
        (Mode::Fit, Some(h)) => fit(&doc, w, file_ref, h.saturating_sub(3)),
        _ => HashSet::new(),
    };
    print!("{}", render(&doc, w, file_ref, &hidden));
}

// ---- tests ----
#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = "\
# Test card
sub: a subtitle
groups: risky=1, safe=9
score: 148/175
note: two at-risk gates in review

## Must Have (x3)
| id | state | score | criterion | note | group |
|----|-------|-------|-----------|------|-------|
| R1 | solid | 5 | Reversible migration | rollback ok | grp:safe |
| R2 | risk  | 3 | Latency budget | see [PR-123](https://example.com/pr/123) | grp:risky |
| R4 | gap   | 0 | Kill switch | untested in prod |

## Callouts
| STANDOUT | R4 is the only red — [details](https://example.com/r4). |
";

    fn empty() -> HashSet<String> {
        HashSet::new()
    }

    #[test]
    fn parses_multiple_group_tags() {
        let d = parse("# t\n## S\n| A | ok | 5 | n | note | grp:x | grp:y |\n");
        let cr = &d.sections[0].crits[0];
        assert_eq!(cr.groups, vec!["x".to_string(), "y".to_string()]);
        assert_eq!(cr.note, "note");
    }

    #[test]
    fn banner_is_in_callouts_group() {
        let d = parse(SAMPLE);
        let b = &d.sections[1].banners[0];
        assert!(banner_membership(b).contains(&"callouts".to_string()));
    }

    #[test]
    fn no_rendered_line_exceeds_width() {
        let d = parse(SAMPLE);
        for file in [None, Some("/Users/x/status.md")] {
            for w in [84usize, 100, 120, 170] {
                for line in render(&d, w, file, &empty()).lines() {
                    assert!(vlen(line) <= w, "width {} > {}: {:?}", vlen(line), w, line);
                }
            }
        }
    }

    #[test]
    fn criterion_columns_align_for_mixed_id_and_score_widths() {
        let d = parse(
            "# t\n## S\n| A | risk | 5 | alpha | first justification |\n| LONG-ID | risk | 100 | beta | second justification |\n",
        );
        let rendered = render(&d, 120, None, &empty());
        let first = rendered
            .lines()
            .find(|line| line.contains("first justification"))
            .unwrap();
        let second = rendered
            .lines()
            .find(|line| line.contains("second justification"))
            .unwrap();

        let column = |line: &str, needle: &str| {
            let byte = line.find(needle).unwrap();
            vlen(&line[..byte])
        };
        assert_eq!(column(first, "AT RISK"), column(second, "AT RISK"));
        assert_eq!(
            column(first, "first justification"),
            column(second, "second justification")
        );
    }

    #[test]
    fn fit_never_drops_higher_priority_before_lower() {
        let s = "# t\ngroups: low=1, high=9\n## S\n| L1 | ok | 5 | lll | n | grp:low |\n| H1 | ok | 5 | hhh | n | grp:high |\n";
        let d = parse(s);
        let full = build_lines(&d, 100, None, &empty()).len();
        for avail in 1..=full {
            let h = fit(&d, 100, None, avail);
            if h.contains("high") {
                assert!(h.contains("low"), "high (p9) dropped before low (p1) at avail {}", avail);
            }
        }
    }

    #[test]
    fn fit_trims_bottom_up_by_default() {
        // no priorities => footer (bottom) hidden before header (top)
        let s = "# t\nfooter: ff\n## S\n| A1 | ok | 5 | aaa | n |\n## Callouts\n| STANDOUT | x |\n";
        let d = parse(s);
        let full = build_lines(&d, 100, None, &empty()).len();
        let hidden = fit(&d, 100, None, full - 1);
        assert!(hidden.contains("footer"));
        assert!(!hidden.contains("header"));
    }

    #[test]
    fn fit_keeps_items_over_chrome() {
        let s = "# t\nscore: 5/10\nfooter: ff\n## S\n| A1 | ok | 5 | aaa | note |\n";
        let d = parse(s);
        let full = build_lines(&d, 100, None, &empty()).len();
        let hidden = fit(&d, 100, None, full - 2);
        assert!(render(&d, 100, None, &hidden).contains("aaa"), "line item must survive");
        assert!(
            hidden.contains("footer") || hidden.contains("meter") || hidden.contains("titles"),
            "chrome should be shed first"
        );
        assert!(!hidden.iter().any(|h| h.starts_with('~')), "no item singleton hidden");
    }

    #[test]
    fn remove_shares_group() {
        let p = std::env::temp_dir().join(format!("scorecard_g_{}.md", std::process::id()));
        let path = p.to_string_lossy().to_string();
        std::fs::write(
            &path,
            "## S\n| A1 | ok | 5 | one | n | grp:g |\n| A2 | gap | 1 | two | n | grp:g |\n| A3 | ok | 5 | three | n |\n",
        )
        .unwrap();
        assert_eq!(remove_line(&path, "A1").unwrap(), 2);
        let after = std::fs::read_to_string(&path).unwrap();
        assert!(!after.contains("| A1 ") && !after.contains("| A2 ") && after.contains("| A3 "));
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn remove_untagged_is_solo() {
        let p = std::env::temp_dir().join(format!("scorecard_s_{}.md", std::process::id()));
        let path = p.to_string_lossy().to_string();
        std::fs::write(&path, "## S\n| A1 | ok | 5 | one | n |\n| A2 | gap | 1 | two | n |\n").unwrap();
        assert_eq!(remove_line(&path, "A1").unwrap(), 1);
        assert!(std::fs::read_to_string(&path).unwrap().contains("| A2 "));
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn percent_roundtrip() {
        let s = "/Users/x y/status.md";
        assert_eq!(percent_decode(&percent_encode(s)), s);
        assert!(percent_encode(s).contains("%20"));
    }

    #[test]
    fn vlen_ignores_sgr_and_osc8() {
        assert_eq!(vlen(&sty(OK, true, "hi")), 2);
        assert_eq!(vlen(&osc8("scorecard://remove/D1?file=/a/b.md", "✕")), 1);
    }

    #[test]
    fn prime_has_content() {
        assert!(PRIME.contains("scorecard") && PRIME.contains("groups:") && PRIME.contains("fit"));
    }

    #[test]
    fn agents_block_upsert_is_idempotent() {
        let b = agents_block();
        let once = upsert_block("# Rules\nfoo\n", &b);
        assert!(once.contains(AGENTS_BEGIN) && once.contains("foo"));
        let twice = upsert_block(&once, &b);
        assert_eq!(once, twice, "re-install must be idempotent");
        assert_eq!(twice.matches(AGENTS_BEGIN).count(), 1);
    }

    #[test]
    fn agents_block_removes_cleanly() {
        let b = agents_block();
        let withb = upsert_block("# Rules\nfoo\n", &b);
        let cleared = remove_block(&withb);
        assert!(!cleared.contains(AGENTS_BEGIN));
        assert!(cleared.contains("foo"));
    }

    #[test]
    fn list_items_emits_markdown() {
        let p = std::env::temp_dir().join(format!("scorecard_list_{}.md", std::process::id()));
        let path = p.to_string_lossy().to_string();
        std::fs::write(&path, "## S\n| D1 | risk | 3 | Ship it | note | grp:x |\n| D2 | ok | 5 | Other | n |\n").unwrap();
        let md = list_items(&path).unwrap();
        assert!(md.contains("`D1`") && md.contains("Ship it") && md.contains("groups: x"));
        assert!(md.contains("`D2`"));
        assert!(md.lines().all(|l| l.is_empty() || l.starts_with("- ")));
        let _ = std::fs::remove_file(&path);
    }
}
