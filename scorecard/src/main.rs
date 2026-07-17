//! scorecard — render a dark, one-screen readiness scorecard TUI from a
//! structured markdown file. Truecolor ANSI, adapts to terminal width,
//! one line per criterion. No external dependencies.
//!
//! Usage:  scorecard [--width N] [--no-actions] <file.md>
//!         scorecard --action scorecard://remove/<id>?file=<path>
//!         scorecard install-handler          # register scorecard:// (macOS)
//!         scorecard uninstall-handler
//!
//! `[text](url)` in the id/note/callout fields renders as an OSC 8 hyperlink.
//! When a file path is known, each row gets a clickable close box (✕) whose
//! link is `scorecard://remove/<id>?file=<path>`; the `scorecard://` scheme is
//! registered (see install-handler.sh) to run `scorecard --action <url>`, which
//! removes that row (matched by its id — the first table column) from the file.

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
}
struct Banner {
    tag: String,
    text: String,
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
    sections: Vec<Section>,
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
                            s.banners.push(Banner {
                                tag: cells[0].clone(),
                                text: cells[1..].join(" ").trim().to_string(),
                            });
                        }
                    }
                    Kind::Crit => {
                        if cells.len() >= 5 {
                            s.crits.push(Crit {
                                id: cells[0].clone(),
                                sev: Sev::from(&cells[1]),
                                score: cells[2].clone(),
                                name: cells[3].clone(),
                                note: cells[4..].join(" | ").trim().to_string(),
                            });
                        } else if cells.len() == 4 {
                            s.crits.push(Crit {
                                id: cells[0].clone(),
                                sev: Sev::from(&cells[1]),
                                score: cells[2].clone(),
                                name: cells[3].clone(),
                                note: String::new(),
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
/// Remove every criteria row whose first cell equals `id`. Returns count.
fn remove_line(file: &str, id: &str) -> Result<usize, String> {
    if !file.ends_with(".md") {
        return Err(format!("refusing to edit non-markdown file: {}", file));
    }
    let content = std::fs::read_to_string(file).map_err(|e| format!("read {}: {}", file, e))?;
    let mut kept: Vec<&str> = Vec::new();
    let mut removed = 0;
    for line in content.lines() {
        let t = line.trim_start();
        if t.starts_with('|') {
            let cells = split_row(t);
            if cells.first().map(|s| s.as_str()) == Some(id) {
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
/// Dispatch a `scorecard://…` action URL. Returns a human message on success.
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
fn notify(msg: &str) {
    let _ = std::process::Command::new("osascript")
        .arg("-e")
        .arg(format!("display notification {:?} with title \"scorecard\"", msg))
        .status();
}

// ---- self-install of the scorecard:// URL-scheme handler (macOS, Route B) ----
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

/// `file` = absolute source path; when Some (and actions enabled), each row gets
/// a clickable ✕ close box linking to `scorecard://remove/<id>?file=…`.
fn render(doc: &Doc, w: usize, file: Option<&str>) -> String {
    let mut cv = Canvas::new(w);
    let iw = cv.iw;

    cv.top();
    if !doc.title.is_empty() {
        cv.boxln(&sty(ACCENT, true, &trunc(&doc.title, iw)));
    }
    if !doc.sub.is_empty() {
        cv.boxln(&sty(MUTED, false, &trunc(&doc.sub, iw)));
    }
    if !doc.meta.is_empty() {
        cv.boxln(&sty(FAINT, false, &trunc(&doc.meta, iw)));
    }
    cv.rule();

    if let Some((proj, max)) = doc.score {
        if max > 0 {
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
                c(FAINT),
                c(TEXT),
                BOLD,
                proj,
                RESET,
                c(FAINT),
                max,
                c(WARN),
                pct,
                RESET,
                bar,
                c(TEXT),
                BOLD,
                c(FAINT),
                pass
            ));
        }
    }
    let (mut n_ok, mut n_warn, mut n_crit, mut n_zero) = (0, 0, 0, 0);
    for s in &doc.sections {
        for cr in &s.crits {
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
    if n_ok + n_warn + n_crit > 0 {
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
    if doc.score.is_some() || n_ok + n_warn + n_crit > 0 {
        cv.rule();
    }

    let mut banner_rule_done = false;
    for s in &doc.sections {
        match s.kind {
            Kind::Crit => {
                if s.crits.is_empty() {
                    continue;
                }
                let hdr = if s.weight.is_empty() {
                    sty(TEXT, true, &s.label)
                } else {
                    format!("{} {}", sty(TEXT, true, &s.label), sty(FAINT, false, &s.weight))
                };
                cv.boxln(&hdr);
                for cr in &s.crits {
                    let close = match file {
                        Some(f) => {
                            let url = format!("scorecard://remove/{}?file={}", cr.id, percent_encode(f));
                            format!(" {}", osc8(&url, &format!("{}✕{}", c(FAINT), RESET)))
                        }
                        None => String::new(),
                    };
                    let cwid = vlen(&close);
                    let id_r = render_field(&cr.id, cv.nw, &format!("{}{}", c(cr.sev.color()), BOLD));
                    let name = pad_end(&trunc(&cr.name, cv.nw), cv.nw);
                    let left = format!(
                        "{}▌{} {} {}{} {} {}{}{}{}",
                        c(cr.sev.color()),
                        RESET,
                        id_r,
                        c(TEXT),
                        name,
                        pill(cr.sev),
                        c(cr.sev.color()),
                        BOLD,
                        cr.score,
                        RESET
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
                if s.banners.is_empty() {
                    continue;
                }
                if !banner_rule_done {
                    cv.rule();
                    banner_rule_done = true;
                }
                for b in &s.banners {
                    let head = sty(banner_color(&b.tag), true, &pad_end(&b.tag, 9));
                    let budget = iw.saturating_sub(vlen(&head));
                    cv.boxln(&format!("{}{}", head, render_field(&b.text, budget, &c(MUTED))));
                }
            }
        }
    }

    cv.rule();
    if !doc.footer.is_empty() {
        cv.boxln(&format!("{}{}{}{}", DIM, c(FAINT), trunc(&doc.footer, iw), RESET));
    }
    cv.bot();

    format!("\n{}\n", cv.lines.join("\n"))
}

fn term_width(cli: Option<usize>) -> usize {
    cli.or_else(|| {
        env::var("COLUMNS")
            .ok()
            .and_then(|s| s.trim().parse::<usize>().ok())
    })
    .unwrap_or(120)
    .clamp(84, 170)
}

fn main() {
    let argv: Vec<String> = env::args().skip(1).collect();
    match argv.first().map(|s| s.as_str()) {
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
    let mut file: Option<String> = None;
    let mut action: Option<String> = None;
    let mut no_actions = false;
    let mut args = argv.into_iter();
    while let Some(a) = args.next() {
        if a == "--width" {
            width_cli = args.next().and_then(|s| s.trim().parse().ok());
        } else if let Some(v) = a.strip_prefix("--width=") {
            width_cli = v.trim().parse().ok();
        } else if a == "--action" {
            action = args.next();
        } else if a == "--no-actions" {
            no_actions = true;
        } else if a == "-h" || a == "--help" {
            eprintln!("usage: scorecard [--width N] [--no-actions] <file.md>");
            eprintln!("       scorecard --action scorecard://remove/<id>?file=<path>");
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
            eprintln!("usage: scorecard [--width N] [--no-actions] <file.md>");
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
    let doc = parse(&input);
    let file_ref = if no_actions { None } else { Some(abs.as_str()) };
    print!("{}", render(&doc, term_width(width_cli), file_ref));
}

// ---- tests ----
#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = "\
# Test card
sub: a subtitle
meta: gate Fri
score: 148/175
pass: 123
note: two at-risk gates in review

## Must Have (x3)
| id | state | score | criterion | note |
|----|-------|-------|-----------|------|
| R1 | solid | 5 | Reversible migration | rollback under 5 min ([runbook](https://example.com/rb)) |
| R2 | risk  | 3 | Latency budget | 210ms vs 200ms; see [PR-123](https://example.com/pr/123) |
| R4 | gap   | 0 | Kill switch | untested in prod |

## Callouts
| STANDOUT | R4 is the only red — [details](https://example.com/r4). |
";

    #[test]
    fn parses_criteria_and_states() {
        let d = parse(SAMPLE);
        let crits: Vec<&Crit> = d.sections.iter().flat_map(|s| s.crits.iter()).collect();
        assert_eq!(crits.len(), 3);
        assert_eq!(crits[0].sev, Sev::Ok);
        assert_eq!(crits[2].sev, Sev::Crit);
    }

    // Width invariant, with links AND close boxes present.
    #[test]
    fn no_rendered_line_exceeds_width() {
        let d = parse(SAMPLE);
        for file in [None, Some("/Users/x/status.md")] {
            for w in [84usize, 100, 120, 170] {
                for line in render(&d, w, file).lines() {
                    assert!(vlen(line) <= w, "width {} > {} (file={:?}): {:?}", vlen(line), w, file, line);
                }
            }
        }
    }

    #[test]
    fn close_box_links_to_remove_action() {
        let d = parse(SAMPLE);
        let out = render(&d, 120, Some("/Users/x/status.md"));
        assert!(out.contains("scorecard://remove/R1?file=/Users/x/status.md"));
    }

    #[test]
    fn vlen_ignores_sgr_and_osc8() {
        assert_eq!(vlen(&sty(OK, true, "hi")), 2);
        assert_eq!(vlen(&osc8("scorecard://remove/D1?file=/a/b.md", "✕")), 1);
    }

    #[test]
    fn parse_links_splits_text_and_link() {
        assert_eq!(
            parse_links("see [IGA-1](https://ex.com/1) now"),
            vec![
                Seg::Text("see ".into()),
                Seg::Link { text: "IGA-1".into(), url: "https://ex.com/1".into() },
                Seg::Text(" now".into()),
            ]
        );
    }

    #[test]
    fn percent_roundtrip() {
        let s = "/Users/x y/status.md";
        assert_eq!(percent_decode(&percent_encode(s)), s);
        assert!(percent_encode(s).contains("%20"));
    }

    #[test]
    fn remove_line_drops_matching_row_only() {
        let p = std::env::temp_dir().join(format!("scorecard_rm_{}.md", std::process::id()));
        let path = p.to_string_lossy().to_string();
        std::fs::write(&path, "## S\n| A1 | ok | 5 | one | n1 |\n| A2 | gap | 1 | two | n2 |\n").unwrap();
        assert_eq!(remove_line(&path, "A1").unwrap(), 1);
        let after = std::fs::read_to_string(&path).unwrap();
        assert!(!after.contains("A1"));
        assert!(after.contains("A2"));
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn remove_line_refuses_non_markdown() {
        assert!(remove_line("/etc/hosts", "x").is_err());
    }

    #[test]
    fn run_action_removes_via_url() {
        let p = std::env::temp_dir().join(format!("scorecard_act_{}.md", std::process::id()));
        let path = p.to_string_lossy().to_string();
        std::fs::write(&path, "## S\n| Z9 | ok | 5 | x | y |\n").unwrap();
        let url = format!("scorecard://remove/Z9?file={}", percent_encode(&path));
        assert!(run_action(&url).is_ok());
        assert!(!std::fs::read_to_string(&path).unwrap().contains("Z9"));
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn run_action_rejects_unknown() {
        assert!(run_action("scorecard://frobnicate/x").is_err());
    }
}
