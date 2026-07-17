//! scorecard — render a dark, one-screen readiness scorecard TUI from a
//! structured markdown file. Truecolor ANSI, adapts to terminal width,
//! one line per criterion. No external dependencies.
//!
//! Usage:  scorecard [--width N] <file.md>
//!         scorecard --width "$COLUMNS" ~/.config/scorecard/status.md
//!
//! Markdown `[text](url)` in the id column, notes, and callouts renders as an
//! OSC 8 terminal hyperlink (clickable in iTerm2/WezTerm/kitty/etc.; plain text
//! elsewhere). See README.md for the schema.

use std::env;
use std::process::exit;

// ---- palette (vaporwave accents; semantics kept distinct from accent) ----
const ACCENT: [u8; 3] = [92, 236, 255]; // cyan — structural accent
const SPARK: [u8; 3] = [255, 0, 153]; // hot pink — spark
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

// ---- visible-length-aware string helpers ----
// Zero-width escapes: SGR/CSI color codes (ESC[ … final-byte) AND OSC 8
// hyperlinks (ESC] … BEL|ST). Both must be skipped or padding/truncation and
// the box alignment break.
fn vlen(s: &str) -> usize {
    let mut n = 0;
    let mut chars = s.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '\x1b' {
            match chars.peek() {
                Some('[') => {
                    chars.next();
                    // CSI: consume through the final byte (0x40..=0x7E, e.g. 'm')
                    while let Some(cc) = chars.next() {
                        if ('@'..='~').contains(&cc) {
                            break;
                        }
                    }
                }
                Some(']') => {
                    chars.next();
                    // OSC: consume through BEL or ST (ESC '\')
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
                    chars.next(); // lone ESC + following byte
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
/// Truncate PLAIN text (no ANSI) to `w` display columns, adding an ellipsis.
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

// ---- inline markdown links -> OSC 8 hyperlinks ----
#[derive(Debug, PartialEq)]
enum Seg {
    Text(String),
    Link { text: String, url: String },
}
fn find_char(chars: &[char], from: usize, target: char) -> Option<usize> {
    (from..chars.len()).find(|&j| chars[j] == target)
}
/// Parse `[text](url)` runs; everything else is literal text.
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
/// Truncate a segment list to `w` VISIBLE columns (a link counts as its text).
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
/// Render a possibly-link-bearing field: apply `base` style, emit OSC 8 for
/// links, truncate to `budget` visible columns, reset at the end.
fn render_field(text: &str, budget: usize, base: &str) -> String {
    let segs = seg_trunc(parse_links(text), budget);
    let mut out = String::from(base);
    for s in &segs {
        match s {
            Seg::Text(t) => out.push_str(t),
            Seg::Link { text, url } => {
                out.push_str(&format!("\x1b]8;;{}\x1b\\{}\x1b]8;;\x1b\\", url, text));
            }
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
    fn bg(self) -> [u8; 3] {
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
        let label = h[..i].trim().to_string();
        let w = h[i + 1..].trim_end_matches(')').trim().to_string();
        (label, w)
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
    let chip = format!("{}{}{} {} {}", bg(sev.bg()), c(sev.color()), BOLD, label, RESET);
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

fn render(doc: &Doc, w: usize) -> String {
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
    let mut n_ok = 0;
    let mut n_warn = 0;
    let mut n_crit = 0;
    let mut n_zero = 0;
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
                    let budget = iw.saturating_sub(vlen(&left) + 1);
                    let note = if budget > 6 && !cr.note.is_empty() {
                        format!(" {}", render_field(&cr.note, budget, &format!("{}{}", DIM, c(MUTED))))
                    } else {
                        String::new()
                    };
                    cv.boxln(&format!("{}{}", left, note));
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
    let mut width_cli: Option<usize> = None;
    let mut file: Option<String> = None;
    let mut args = env::args().skip(1);
    while let Some(a) = args.next() {
        if a == "--width" {
            width_cli = args.next().and_then(|s| s.trim().parse().ok());
        } else if let Some(v) = a.strip_prefix("--width=") {
            width_cli = v.trim().parse().ok();
        } else if a == "-h" || a == "--help" {
            eprintln!("usage: scorecard [--width N] <file.md>");
            exit(0);
        } else if !a.starts_with('-') {
            file = Some(a);
        }
    }

    let path = match file {
        Some(p) => p,
        None => {
            eprintln!("usage: scorecard [--width N] <file.md>");
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
    let doc = parse(&input);
    print!("{}", render(&doc, term_width(width_cli)));
}

// ---- tests ----
#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = "\
# Test card
sub: a subtitle
meta: gate Fri  ·  weights must x3
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
    fn parses_front_matter_and_score() {
        let d = parse(SAMPLE);
        assert_eq!(d.title, "Test card");
        assert_eq!(d.score, Some((148, 175)));
        assert_eq!(d.pass, Some(123));
    }

    #[test]
    fn parses_criteria_and_states() {
        let d = parse(SAMPLE);
        let crits: Vec<&Crit> = d.sections.iter().flat_map(|s| s.crits.iter()).collect();
        assert_eq!(crits.len(), 3);
        assert_eq!(crits[0].sev, Sev::Ok);
        assert_eq!(crits[1].sev, Sev::Warn);
        assert_eq!(crits[2].sev, Sev::Crit);
        assert_eq!(crits[2].score, "0");
    }

    #[test]
    fn sev_mapping() {
        assert_eq!(Sev::from("SOLID"), Sev::Ok);
        assert_eq!(Sev::from("at-risk"), Sev::Warn);
        assert_eq!(Sev::from("gap"), Sev::Crit);
    }

    // The bug class we hit in the prototype: a line wider than the box —
    // now including OSC 8 hyperlinks in notes and callouts.
    #[test]
    fn no_rendered_line_exceeds_width() {
        let d = parse(SAMPLE);
        for w in [84usize, 100, 120, 170] {
            for line in render(&d, w).lines() {
                assert!(vlen(line) <= w, "line width {} > {}: {:?}", vlen(line), w, line);
            }
        }
    }

    #[test]
    fn vlen_ignores_sgr() {
        assert_eq!(vlen(&sty(OK, true, "hello")), 5);
        assert_eq!(vlen("plain"), 5);
    }

    #[test]
    fn vlen_ignores_osc8_hyperlink() {
        let link = "\x1b]8;;https://example.com/very/long/path\x1b\\IGA-1\x1b]8;;\x1b\\";
        assert_eq!(vlen(link), 5); // "IGA-1"
    }

    #[test]
    fn parse_links_splits_text_and_link() {
        let segs = parse_links("see [IGA-1](https://ex.com/1) now");
        assert_eq!(
            segs,
            vec![
                Seg::Text("see ".into()),
                Seg::Link {
                    text: "IGA-1".into(),
                    url: "https://ex.com/1".into()
                },
                Seg::Text(" now".into()),
            ]
        );
    }

    #[test]
    fn parse_links_leaves_plain_brackets() {
        assert_eq!(parse_links("[not a link]"), vec![Seg::Text("[not a link]".into())]);
    }

    #[test]
    fn seg_vlen_counts_link_text_only() {
        let segs = parse_links("[IGA-1](https://very-long-url.example.com/abcdef) tail");
        assert_eq!(seg_vlen(&segs), "IGA-1 tail".chars().count());
    }

    #[test]
    fn render_field_link_is_measured_by_text() {
        let out = render_field("go [x](https://e.com/averylongpath) end", 100, "");
        assert!(out.contains("\x1b]8;;https://e.com/averylongpath\x1b\\x\x1b]8;;\x1b\\"));
        assert_eq!(vlen(&out), "go x end".chars().count());
    }

    #[test]
    fn trunc_adds_ellipsis() {
        assert_eq!(trunc("hello world", 5), "hell…");
        assert_eq!(trunc("hi", 5), "hi");
    }
}
