//! render-last — render block LaTeX math from an agent's last assistant
//! message as inline iTerm2 images (OSC 1337), so the terminal shows typeset
//! math instead of raw markup. Std-only: shells out to `jq`, `tectonic`, and
//! `pdftocairo`.
//!
//! Usage:
//!   render-last                    # default: resolve + render the current
//!                                   # Claude Code session's last assistant
//!                                   # message
//!   render-last --file PATH        # read a transcript from PATH instead
//!   render-last --session UUID     # read <UUID>.jsonl from the resolved
//!                                   # transcript directory
//!   render-last - | --stdin        # read raw markdown from stdin (pipe
//!                                   # mode: `agent -p | render-last -`)
//!   render-last --list             # print detected block-math spans and a
//!                                   # count, render nothing (alias:
//!                                   # --dry-run)
//!   render-last --color NAME|HEX   # pink|cyan|magenta|gold|purple, or a
//!                                   # raw 6-hex value (default: cyan)
//!   render-last --dpi N            # rasterization DPI (default: 130)
//!
//! Only block math is detected: `$$ ... $$`, `\[ ... \]`, and fenced
//! ```math / ```latex blocks. Inline `$ ... $` and `\( ... \)` are left as
//! plain text. Delimiters inside a fenced code block (any info string other
//! than `math`/`latex`) or inline code span never count — a bash `[ -f x ]`
//! test or an ANSI regex inside a ``` fence is prose, not math. A `$$`/`\[`
//! match longer than 1500 bytes or spanning a blank line is rejected as
//! implausible display math and left as plain text.
//!
//! Transcript resolution (default mode): mangle $PWD by replacing every `/`
//! with `-` (a leading `/` becomes a leading `-`), look under
//! `~/.claude/projects/<mangled>/`, and pick the most-recently-modified
//! `*.jsonl` there. The last assistant text message in that transcript is
//! extracted via a streaming `jq` filter (never `jq -s`: these transcripts
//! run tens of MB and slurping is slow and can fail).
//!
//! A block that fails to compile is left in the output as raw LaTeX plus a
//! ` [render failed] ` marker; the rest of the message still renders.

use std::env;
use std::fs;
use std::io::{Read as _, Write as _};
use std::path::{Path, PathBuf};
use std::process::{exit, Command, Stdio};

// ---------------------------------------------------------------------
// CLI
// ---------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
enum Source {
    DefaultTranscript,
    File(String),
    Session(String),
    Stdin,
}

/// How to color rendered math: rotate through the palette, or pin one color.
#[derive(Debug, Clone, PartialEq, Eq)]
enum ColorMode {
    Rotate,
    Fixed(String),
}

#[derive(Debug, PartialEq)]
struct Args {
    source: Source,
    color: ColorMode,
    dpi: u32,
    list: bool,
}

fn parse_args(args: &[String]) -> Result<Args, String> {
    let mut source = Source::DefaultTranscript;
    let mut color = ColorMode::Rotate;
    let mut dpi: u32 = 130;
    let mut list = false;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "-" | "--stdin" => source = Source::Stdin,
            "--file" => {
                i += 1;
                let path = args.get(i).ok_or("--file requires a path")?;
                source = Source::File(path.clone());
            }
            "--session" => {
                i += 1;
                let uuid = args.get(i).ok_or("--session requires a uuid")?;
                source = Source::Session(uuid.clone());
            }
            "--color" => {
                i += 1;
                let value = args.get(i).ok_or("--color requires a value")?;
                color = ColorMode::Fixed(value.clone());
            }
            "--rotate" => color = ColorMode::Rotate,
            "--dpi" => {
                i += 1;
                let raw = args.get(i).ok_or("--dpi requires a value")?;
                dpi = raw
                    .parse::<u32>()
                    .map_err(|_| format!("--dpi: invalid number: {raw}"))?;
            }
            "--list" | "--dry-run" => list = true,
            other => return Err(format!("unrecognized argument: {other}")),
        }
        i += 1;
    }
    Ok(Args {
        source,
        color,
        dpi,
        list,
    })
}

// ---------------------------------------------------------------------
// color
// ---------------------------------------------------------------------

/// Resolve a `--color` value to a lowercase 6-hex-digit string: a palette
/// name, or a raw hex value (with or without a leading `#`). None on
/// anything else.
fn resolve_color(input: &str) -> Option<String> {
    match input.to_lowercase().as_str() {
        "pink" => return Some("ff0099".to_string()),
        "cyan" => return Some("5cecff".to_string()),
        "magenta" => return Some("ff00f8".to_string()),
        "gold" => return Some("fbb725".to_string()),
        "purple" => return Some("aa00e8".to_string()),
        _ => {}
    }
    let hex = input.strip_prefix('#').unwrap_or(input);
    if hex.len() == 6 && hex.bytes().all(|b| b.is_ascii_hexdigit()) {
        Some(hex.to_lowercase())
    } else {
        None
    }
}

/// The vaporwave palette in rotation order: pink, cyan, magenta, gold, purple.
const PALETTE: [&str; 5] = ["ff0099", "5cecff", "ff00f8", "fbb725", "aa00e8"];

/// `n` palette hexes starting at index `start` (wrapping) — one per math
/// block, so each equation in a message gets the next color.
fn rotate_colors(start: usize, n: usize) -> Vec<String> {
    (0..n)
        .map(|j| PALETTE[(start + j) % PALETTE.len()].to_string())
        .collect()
}

fn rotate_state_path() -> PathBuf {
    home_dir().join(".local/state/render-last/rotate-index")
}

/// Persisted rotation offset, so successive invocations keep cycling instead
/// of restarting at pink each time. Absent or garbled reads as 0.
fn read_rotate_start() -> usize {
    fs::read_to_string(rotate_state_path())
        .ok()
        .and_then(|s| s.trim().parse::<usize>().ok())
        .map(|v| v % PALETTE.len())
        .unwrap_or(0)
}

fn write_rotate_start(next: usize) {
    let path = rotate_state_path();
    if let Some(dir) = path.parent() {
        let _ = fs::create_dir_all(dir);
    }
    let _ = fs::write(&path, (next % PALETTE.len()).to_string());
}

// ---------------------------------------------------------------------
// block math detection
// ---------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
struct Span {
    start: usize,
    end: usize,
    latex: String,
}

/// Byte offset of the next char boundary after `i`, so the scan in
/// `find_block_math` never slices `md` on a non-UTF8-boundary index.
fn next_boundary(s: &str, i: usize) -> usize {
    match s[i..].chars().next() {
        Some(c) => i + c.len_utf8(),
        None => i + 1,
    }
}

/// Reject a `$$`/`\[` match that is too long or crosses a blank line to be
/// plausible display math — a stray delimiter re-pairing with an unrelated
/// one much later in the message.
const MAX_DISPLAY_MATH_LEN: usize = 1500;

fn is_plausible_display_math(raw_inner: &str) -> bool {
    raw_inner.len() <= MAX_DISPLAY_MATH_LEN && !raw_inner.contains("\n\n")
}

/// A fenced code block (3+ backticks) at the start of `rest`: whether its
/// info string is `math`/`latex`, and the byte offset (from `rest`'s start)
/// where the fenced body begins.
fn fence_open(rest: &str) -> Option<(bool, usize)> {
    let after_ticks = rest.strip_prefix("```")?;
    let info_line_end = after_ticks.find('\n').unwrap_or(after_ticks.len());
    let info = after_ticks[..info_line_end].trim();
    let is_math = info.eq_ignore_ascii_case("math") || info.eq_ignore_ascii_case("latex");
    let body_offset = 3 + if info_line_end < after_ticks.len() {
        info_line_end + 1
    } else {
        info_line_end
    };
    Some((is_math, body_offset))
}

/// An inline code span (1 or 2 backticks, not a fence) at the start of
/// `rest`: the byte length to skip past its closing delimiter, if one
/// exists.
fn inline_code_skip_len(rest: &str) -> Option<usize> {
    let open_len = rest.bytes().take_while(|&b| b == b'`').count();
    if open_len == 0 || open_len >= 3 {
        return None;
    }
    let marker = "`".repeat(open_len);
    rest[open_len..].find(&marker).map(|rel| open_len + rel + open_len)
}

/// Find block-math spans: `$$ ... $$`, `\[ ... \]`, and fenced ```math /
/// ```latex blocks. Inline `$ ... $` and `\( ... \)` are never matched — a
/// lone `$` only becomes a match if immediately followed by a second `$`.
/// Content inside a non-math fenced block or an inline code span is opaque:
/// delimiters there are skipped over, never scanned. Unterminated
/// delimiters (no matching close) are left as plain text.
fn find_block_math(md: &str) -> Vec<Span> {
    let mut spans = Vec::new();
    let mut i = 0;
    while i < md.len() {
        if let Some(rest) = md[i..].strip_prefix("$$") {
            if let Some(rel_end) = rest.find("$$") {
                let inner_start = i + 2;
                let inner_end = inner_start + rel_end;
                let end = inner_end + 2;
                let raw = &md[inner_start..inner_end];
                if is_plausible_display_math(raw) {
                    spans.push(Span {
                        start: i,
                        end,
                        latex: raw.trim().to_string(),
                    });
                    i = end;
                    continue;
                }
            }
        } else if let Some(rest) = md[i..].strip_prefix("\\[") {
            if let Some(rel_end) = rest.find("\\]") {
                let inner_start = i + 2;
                let inner_end = inner_start + rel_end;
                let end = inner_end + 2;
                let raw = &md[inner_start..inner_end];
                if is_plausible_display_math(raw) {
                    spans.push(Span {
                        start: i,
                        end,
                        latex: raw.trim().to_string(),
                    });
                    i = end;
                    continue;
                }
            }
        } else if let Some((is_math, body_offset)) = fence_open(&md[i..]) {
            let body_start = i + body_offset;
            if let Some(rel_end) = md[body_start..].find("```") {
                let inner_end = body_start + rel_end;
                let end = inner_end + 3;
                if is_math {
                    spans.push(Span {
                        start: i,
                        end,
                        latex: md[body_start..inner_end].trim().to_string(),
                    });
                }
                i = end;
                continue;
            }
        } else if md[i..].starts_with('`') {
            if let Some(skip_len) = inline_code_skip_len(&md[i..]) {
                i += skip_len;
                continue;
            }
        }
        i = next_boundary(md, i);
    }
    spans
}

fn print_list(spans: &[Span]) {
    println!("{} block math span(s) found", spans.len());
    for (index, span) in spans.iter().enumerate() {
        let preview: String = span.latex.chars().take(60).collect();
        println!("{index}: {preview}");
    }
}

// ---------------------------------------------------------------------
// LaTeX wrapping
// ---------------------------------------------------------------------

/// Wrap `inner` in a standalone LaTeX document colored `hex6`. If `inner`
/// starts with `\begin{` (an environment such as align or matrix) it is
/// used as-is; otherwise it is wrapped in `\[ ... \]`.
fn wrap_latex(inner: &str, hex6: &str) -> String {
    let body = if inner.starts_with("\\begin{") {
        inner.to_string()
    } else {
        format!("\\[ {inner} \\]")
    };
    format!(
        "\\documentclass[border=6pt,varwidth]{{standalone}}\n\
         \\usepackage{{amsmath,amssymb}}\n\
         \\usepackage{{xcolor}}\n\
         \\begin{{document}}\n\
         \\color[HTML]{{{hex6}}}\n\
         {body}\n\
         \\end{{document}}\n"
    )
}

// ---------------------------------------------------------------------
// base64 + iTerm2 inline image escape
// ---------------------------------------------------------------------

const BASE64_ALPHABET: &[u8; 64] =
    b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

/// Hand-rolled RFC 4648 base64 encoder (standard alphabet, `=` padding).
fn base64_encode(data: &[u8]) -> String {
    let mut out = String::with_capacity(data.len().div_ceil(3) * 4);
    for chunk in data.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = *chunk.get(1).unwrap_or(&0) as u32;
        let b2 = *chunk.get(2).unwrap_or(&0) as u32;
        let n = (b0 << 16) | (b1 << 8) | b2;
        out.push(BASE64_ALPHABET[((n >> 18) & 0x3f) as usize] as char);
        out.push(BASE64_ALPHABET[((n >> 12) & 0x3f) as usize] as char);
        out.push(if chunk.len() > 1 {
            BASE64_ALPHABET[((n >> 6) & 0x3f) as usize] as char
        } else {
            '='
        });
        out.push(if chunk.len() > 2 {
            BASE64_ALPHABET[(n & 0x3f) as usize] as char
        } else {
            '='
        });
    }
    out
}

/// Build an iTerm2 inline-image OSC 1337 escape sequence for a PNG's raw
/// bytes.
fn iterm_inline_image_escape(png_bytes: &[u8]) -> String {
    format!(
        "\x1b]1337;File=inline=1;preserveAspectRatio=1;size={}:{}\x07",
        png_bytes.len(),
        base64_encode(png_bytes)
    )
}

// ---------------------------------------------------------------------
// transcript resolution + extraction
// ---------------------------------------------------------------------

fn home_dir() -> PathBuf {
    PathBuf::from(env::var("HOME").expect("HOME not set"))
}

/// Current working directory as a string, preferring `$PWD` (matches the
/// shell's notion of cwd, including through symlinks) and falling back to
/// the process's own cwd.
fn cwd_string() -> String {
    env::var("PWD")
        .ok()
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| {
            env::current_dir()
                .map(|p| p.to_string_lossy().into_owned())
                .unwrap_or_default()
        })
}

/// /Users/rch/repo/x -> -Users-rch-repo-x
fn mangle_cwd(cwd: &str) -> String {
    cwd.replace('/', "-")
}

fn transcript_dir() -> PathBuf {
    home_dir()
        .join(".claude/projects")
        .join(mangle_cwd(&cwd_string()))
}

/// The most-recently-modified `*.jsonl` transcript in the resolved
/// transcript directory.
fn resolve_default_transcript() -> Result<PathBuf, String> {
    let dir = transcript_dir();
    let entries = fs::read_dir(&dir)
        .map_err(|e| format!("no transcript directory {}: {e}", dir.display()))?;
    let mut best: Option<(PathBuf, std::time::SystemTime)> = None;
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("jsonl") {
            continue;
        }
        let Ok(meta) = entry.metadata() else {
            continue;
        };
        let Ok(mtime) = meta.modified() else {
            continue;
        };
        if best.as_ref().is_none_or(|(_, best_mtime)| mtime > *best_mtime) {
            best = Some((path, mtime));
        }
    }
    best.map(|(path, _)| path)
        .ok_or_else(|| format!("no *.jsonl transcript found in {}", dir.display()))
}

/// Stream the transcript through `jq`, extracting each assistant message's
/// joined text content as one compact JSON string per line, and decode the
/// last one. Never slurps the whole file (`jq -s`): these transcripts run
/// tens of MB.
const JQ_LAST_ASSISTANT_TEXT_FILTER: &str = r#"select(.type=="assistant") | [.message.content[]? | select(.type=="text") | .text] | join("\n") | select(. != "")"#;

fn extract_last_assistant_text(transcript: &Path) -> Result<String, String> {
    let output = Command::new("jq")
        .arg("-c")
        .arg(JQ_LAST_ASSISTANT_TEXT_FILTER)
        .arg(transcript)
        .output()
        .map_err(|e| format!("failed to run jq: {e}"))?;
    if !output.status.success() {
        return Err(format!(
            "jq failed on {}: {}",
            transcript.display(),
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    let last_line = stdout
        .lines()
        .next_back()
        .ok_or_else(|| format!("no assistant text found in {}", transcript.display()))?;
    decode_json_string_via_jq(last_line)
}

/// Decode one jq-emitted JSON string (`"foo\nbar"`) back to raw text, via
/// `jq -r .` rather than a hand-rolled unescaper.
fn decode_json_string_via_jq(line: &str) -> Result<String, String> {
    let mut child = Command::new("jq")
        .arg("-r")
        .arg(".")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("failed to run jq -r .: {e}"))?;
    child
        .stdin
        .take()
        .expect("stdin was piped")
        .write_all(line.as_bytes())
        .map_err(|e| format!("failed to write to jq: {e}"))?;
    let output = child
        .wait_with_output()
        .map_err(|e| format!("failed to wait for jq: {e}"))?;
    if !output.status.success() {
        return Err(format!(
            "jq -r . failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    Ok(String::from_utf8_lossy(&output.stdout)
        .trim_end_matches('\n')
        .to_string())
}

fn load_markdown(source: &Source) -> Result<String, String> {
    match source {
        Source::Stdin => {
            let mut buf = String::new();
            std::io::stdin()
                .read_to_string(&mut buf)
                .map_err(|e| format!("failed to read stdin: {e}"))?;
            Ok(buf)
        }
        Source::File(path) => extract_last_assistant_text(Path::new(path)),
        Source::Session(uuid) => {
            extract_last_assistant_text(&transcript_dir().join(format!("{uuid}.jsonl")))
        }
        Source::DefaultTranscript => {
            extract_last_assistant_text(&resolve_default_transcript()?)
        }
    }
}

// ---------------------------------------------------------------------
// rendering: LaTeX -> PDF -> PNG -> OSC 1337
// ---------------------------------------------------------------------

fn tool_on_path(name: &str) -> bool {
    Command::new(name).arg("--version").output().is_ok()
}

fn check_tools_available() {
    let missing: Vec<&str> = [("tectonic", "tectonic"), ("pdftocairo", "pdftocairo")]
        .into_iter()
        .filter(|(_, bin)| !tool_on_path(bin))
        .map(|(name, _)| name)
        .collect();
    if !missing.is_empty() {
        eprintln!(
            "render-last: missing required tool(s): {}. Install with: brew install tectonic poppler",
            missing.join(", ")
        );
        exit(1);
    }
}

/// First-ever `tectonic` run downloads the TeX bundle, which is slow. Print
/// a one-line stderr hint if the local bundle cache looks unpopulated.
fn maybe_print_first_run_hint() {
    if !home_dir().join(".cache/Tectonic").is_dir() {
        eprintln!("render-last: first tectonic run downloads the TeX bundle, this may take a while...");
    }
}

fn make_run_dir() -> Result<PathBuf, String> {
    let unique = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let dir = env::temp_dir().join(format!("render-last-{}-{unique}", std::process::id()));
    fs::create_dir_all(&dir).map_err(|e| format!("failed to create {}: {e}", dir.display()))?;
    Ok(dir)
}

/// Render one block of LaTeX to an iTerm2 inline-image escape sequence:
/// wrap it in a standalone document, compile with `tectonic`, rasterize the
/// PDF to a transparent PNG with `pdftocairo`, base64-encode it.
fn render_block(latex: &str, hex6: &str, dpi: u32) -> Result<String, String> {
    let run_dir = make_run_dir()?;
    let tex_path = run_dir.join("block.tex");
    fs::write(&tex_path, wrap_latex(latex, hex6))
        .map_err(|e| format!("failed to write {}: {e}", tex_path.display()))?;

    let result = render_block_in_dir(&run_dir, &tex_path, dpi);
    let _ = fs::remove_dir_all(&run_dir);
    result
}

fn render_block_in_dir(run_dir: &Path, tex_path: &Path, dpi: u32) -> Result<String, String> {
    let tectonic_out = Command::new("tectonic")
        .arg("-o")
        .arg(run_dir)
        .arg(tex_path)
        .output()
        .map_err(|e| format!("failed to run tectonic: {e}"))?;
    if !tectonic_out.status.success() {
        return Err(format!(
            "tectonic failed: {}",
            String::from_utf8_lossy(&tectonic_out.stderr)
                .lines()
                .next_back()
                .unwrap_or("")
        ));
    }

    let pdf_path = run_dir.join("block.pdf");
    let prefix = run_dir.join("block");
    let cairo_out = Command::new("pdftocairo")
        .arg("-png")
        .arg("-transp")
        .arg("-r")
        .arg(dpi.to_string())
        .arg(&pdf_path)
        .arg(&prefix)
        .output()
        .map_err(|e| format!("failed to run pdftocairo: {e}"))?;
    if !cairo_out.status.success() {
        return Err(format!(
            "pdftocairo failed: {}",
            String::from_utf8_lossy(&cairo_out.stderr).trim()
        ));
    }

    let png_path = run_dir.join("block-1.png");
    let png_bytes = fs::read(&png_path)
        .map_err(|e| format!("failed to read {}: {e}", png_path.display()))?;
    Ok(iterm_inline_image_escape(&png_bytes))
}

fn render_message(markdown: &str, spans: &[Span], colors: &[String], dpi: u32) -> String {
    let mut out = String::with_capacity(markdown.len());
    let mut last = 0;
    for (idx, span) in spans.iter().enumerate() {
        out.push_str(&markdown[last..span.start]);
        match render_block(&span.latex, &colors[idx], dpi) {
            Ok(escape) => {
                out.push('\n');
                out.push_str(&escape);
                out.push('\n');
            }
            Err(msg) => {
                eprintln!("render-last: block render failed: {msg}");
                out.push_str(&markdown[span.start..span.end]);
                out.push_str(" [render failed] ");
            }
        }
        last = span.end;
    }
    out.push_str(&markdown[last..]);
    out
}

// ---------------------------------------------------------------------
// main
// ---------------------------------------------------------------------

fn main() {
    let raw_args: Vec<String> = env::args().skip(1).collect();
    let args = match parse_args(&raw_args) {
        Ok(a) => a,
        Err(e) => {
            eprintln!("render-last: {e}");
            exit(1);
        }
    };

    // Validate a pinned color up front; rotation needs no validation.
    if let ColorMode::Fixed(c) = &args.color {
        if resolve_color(c).is_none() {
            eprintln!("render-last: unrecognized --color value: {c}");
            exit(1);
        }
    }

    let markdown = match load_markdown(&args.source) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("render-last: {e}");
            exit(1);
        }
    };

    let spans = find_block_math(&markdown);

    if args.list {
        print_list(&spans);
        return;
    }

    if spans.is_empty() {
        print!("{markdown}");
        return;
    }

    let colors: Vec<String> = match &args.color {
        ColorMode::Fixed(c) => vec![resolve_color(c).unwrap(); spans.len()],
        ColorMode::Rotate => {
            let start = read_rotate_start();
            write_rotate_start(start + spans.len());
            rotate_colors(start, spans.len())
        }
    };

    check_tools_available();
    maybe_print_first_run_hint();
    print!("{}", render_message(&markdown, &spans, &colors, args.dpi));
}

// ---------------------------------------------------------------------
// tests
// ---------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // --- resolve_color ---

    #[test]
    fn resolve_color_maps_palette_names_case_insensitively() {
        assert_eq!(resolve_color("pink").as_deref(), Some("ff0099"));
        assert_eq!(resolve_color("CYAN").as_deref(), Some("5cecff"));
        assert_eq!(resolve_color("Magenta").as_deref(), Some("ff00f8"));
        assert_eq!(resolve_color("gold").as_deref(), Some("fbb725"));
        assert_eq!(resolve_color("purple").as_deref(), Some("aa00e8"));
    }

    #[test]
    fn resolve_color_passes_through_raw_hex_with_or_without_hash() {
        assert_eq!(resolve_color("ABCDEF").as_deref(), Some("abcdef"));
        assert_eq!(resolve_color("#abcdef").as_deref(), Some("abcdef"));
    }

    #[test]
    fn resolve_color_rejects_junk() {
        assert_eq!(resolve_color("notacolor"), None);
        assert_eq!(resolve_color("12345"), None); // too short
        assert_eq!(resolve_color("gggggg"), None); // not hex digits
    }

    #[test]
    fn rotate_colors_cycles_the_palette_from_the_start_offset() {
        assert_eq!(rotate_colors(0, 1), vec!["ff0099".to_string()]); // pink first
        assert_eq!(
            rotate_colors(0, 6),
            ["ff0099", "5cecff", "ff00f8", "fbb725", "aa00e8", "ff0099"]
                .iter()
                .map(|s| s.to_string())
                .collect::<Vec<_>>()
        );
        // wraps and starts at the given offset
        assert_eq!(
            rotate_colors(4, 2),
            vec!["aa00e8".to_string(), "ff0099".to_string()]
        );
        assert!(rotate_colors(0, 0).is_empty());
    }

    // --- find_block_math ---

    #[test]
    fn find_block_math_ignores_inline_dollar_math() {
        assert!(find_block_math("here is $x$ inline").is_empty());
    }

    #[test]
    fn find_block_math_ignores_inline_paren_math() {
        assert!(find_block_math(r"here is \(x\) inline").is_empty());
    }

    #[test]
    fn find_block_math_finds_double_dollar_block() {
        let spans = find_block_math("text $$x^2 + y^2$$ more");
        assert_eq!(spans.len(), 1);
        assert_eq!(spans[0].latex, "x^2 + y^2");
        assert_eq!(&"text $$x^2 + y^2$$ more"[spans[0].start..spans[0].end], "$$x^2 + y^2$$");
    }

    #[test]
    fn find_block_math_finds_bracket_block() {
        let spans = find_block_math(r"before \[ a + b \] after");
        assert_eq!(spans.len(), 1);
        assert_eq!(spans[0].latex, "a + b");
    }

    #[test]
    fn find_block_math_finds_math_fence() {
        let spans = find_block_math("```math\nx = 1\n```");
        assert_eq!(spans.len(), 1);
        assert_eq!(spans[0].latex, "x = 1");
    }

    #[test]
    fn find_block_math_finds_latex_fence() {
        let spans = find_block_math("```latex\ny = 2\n```");
        assert_eq!(spans.len(), 1);
        assert_eq!(spans[0].latex, "y = 2");
    }

    #[test]
    fn find_block_math_finds_multiple_adjacent_blocks() {
        let spans = find_block_math("$$a$$$$b$$");
        assert_eq!(spans.len(), 2);
        assert_eq!(spans[0].latex, "a");
        assert_eq!(spans[1].latex, "b");
    }

    #[test]
    fn find_block_math_finds_multiple_scattered_blocks_of_different_kinds() {
        let md = "one $$a$$ two \\[ b \\] three ```math\nc\n``` four";
        let spans = find_block_math(md);
        assert_eq!(spans.len(), 3);
        assert_eq!(spans[0].latex, "a");
        assert_eq!(spans[1].latex, "b");
        assert_eq!(spans[2].latex, "c");
    }

    #[test]
    fn find_block_math_handles_multibyte_text_without_panicking() {
        let spans = find_block_math("emoji 🎉 text $$x$$ more emoji 🎉");
        assert_eq!(spans.len(), 1);
        assert_eq!(spans[0].latex, "x");
    }

    // Failure hypothesis: a naive scanner matches `\[`/`\]` wherever they
    // appear, including inside a bash test or a regex pasted in a code
    // fence — neither is math.
    #[test]
    fn find_block_math_ignores_bracket_delimiters_inside_a_code_fence() {
        let md = "```bash\nif [ -f x ]; then\n  \\[ -f x \\]\nfi\n```";
        assert!(find_block_math(md).is_empty());
    }

    #[test]
    fn find_block_math_ignores_regex_brackets_inside_a_code_fence() {
        let md = "```js\nconst ANSI = /\\x1b\\[[0-9;]*m/;\n```";
        assert!(find_block_math(md).is_empty());
    }

    #[test]
    fn find_block_math_ignores_dollar_and_bracket_delimiters_inside_inline_code() {
        assert!(find_block_math("run `\\[ a \\]` in prose").is_empty());
        assert!(find_block_math("run `$$a$$` in prose").is_empty());
    }

    #[test]
    fn find_block_math_still_finds_real_math_alongside_ignored_code() {
        let md = "```bash\n\\[ -f x \\]\n```\nand here is real math \\[ x^2 \\] done";
        let spans = find_block_math(md);
        assert_eq!(spans.len(), 1);
        assert_eq!(spans[0].latex, "x^2");
    }

    // Failure hypothesis: an unterminated or wildly mismatched delimiter
    // re-pairs with an unrelated one far later in the message, capturing
    // huge unrelated content as "latex".
    #[test]
    fn find_block_math_rejects_span_over_length_guard() {
        let overlong = "x".repeat(MAX_DISPLAY_MATH_LEN + 1);
        let md = format!("$${overlong}$$");
        assert!(find_block_math(&md).is_empty());
    }

    #[test]
    fn find_block_math_rejects_span_crossing_a_blank_line() {
        let md = "$$a\n\nb$$";
        assert!(find_block_math(md).is_empty());
    }

    #[test]
    fn find_block_math_accepts_span_at_the_length_guard_boundary() {
        let exact = "x".repeat(MAX_DISPLAY_MATH_LEN);
        let md = format!("$${exact}$$");
        assert_eq!(find_block_math(&md).len(), 1);
    }

    // --- wrap_latex ---

    #[test]
    fn wrap_latex_includes_the_hex_color() {
        let doc = wrap_latex("x^2", "5cecff");
        assert!(doc.contains("\\color[HTML]{5cecff}"));
    }

    #[test]
    fn wrap_latex_wraps_plain_expressions_in_display_brackets() {
        let doc = wrap_latex("x^2", "ffffff");
        assert!(doc.contains("\\[ x^2 \\]"));
    }

    #[test]
    fn wrap_latex_passes_environments_through_unwrapped() {
        let doc = wrap_latex("\\begin{align}x &= 1\\end{align}", "ffffff");
        assert!(doc.contains("\\begin{align}x &= 1\\end{align}"));
        assert!(!doc.contains("\\[ \\begin"));
    }

    // --- base64_encode (RFC 4648 test vectors) ---

    #[test]
    fn base64_encode_matches_rfc4648_test_vectors() {
        assert_eq!(base64_encode(b""), "");
        assert_eq!(base64_encode(b"f"), "Zg==");
        assert_eq!(base64_encode(b"fo"), "Zm8=");
        assert_eq!(base64_encode(b"foo"), "Zm9v");
        assert_eq!(base64_encode(b"foob"), "Zm9vYg==");
        assert_eq!(base64_encode(b"fooba"), "Zm9vYmE=");
        assert_eq!(base64_encode(b"foobar"), "Zm9vYmFy");
    }

    // --- iterm_inline_image_escape ---

    #[test]
    fn iterm_inline_image_escape_has_expected_prefix_and_suffix() {
        let escape = iterm_inline_image_escape(b"hello");
        assert!(escape.starts_with("\x1b]1337;File=inline=1;preserveAspectRatio=1;size=5:"));
        assert!(escape.ends_with('\x07'));
    }

    #[test]
    fn iterm_inline_image_escape_embeds_correct_base64_payload() {
        let escape = iterm_inline_image_escape(b"foobar");
        assert!(escape.contains(&base64_encode(b"foobar")));
    }

    // --- mangle_cwd ---

    #[test]
    fn mangle_cwd_replaces_all_slashes() {
        assert_eq!(mangle_cwd("/Users/rch/repo/x"), "-Users-rch-repo-x");
    }

    // --- transcript extraction ---

    fn write_synthetic_transcript(label: &str) -> PathBuf {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = env::temp_dir().join(format!(
            "render-last-test-{label}-{}-{unique}",
            std::process::id()
        ));
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("transcript.jsonl");
        let content = concat!(
            r#"{"type":"user","message":{"content":[{"type":"text","text":"question one"}]}}"#,
            "\n",
            r#"{"type":"assistant","message":{"content":[{"type":"text","text":"first answer"}]}}"#,
            "\n",
            r#"{"type":"user","message":{"content":[{"type":"text","text":"question two"}]}}"#,
            "\n",
            r#"{"type":"assistant","message":{"content":[{"type":"text","text":"second answer"},{"type":"tool_use","name":"x"}]}}"#,
            "\n",
            r#"{"type":"assistant","message":{"content":[{"type":"text","text":"final answer"}]}}"#,
            "\n",
        );
        fs::write(&path, content).unwrap();
        path
    }

    // Failure hypothesis: extraction concatenates every assistant message in
    // the file (or grabs the first one) instead of isolating the last.
    #[test]
    fn extract_last_assistant_text_returns_only_the_final_message() {
        let path = write_synthetic_transcript("extract");
        let result = extract_last_assistant_text(&path).unwrap();
        let _ = fs::remove_dir_all(path.parent().unwrap());
        assert_eq!(result, "final answer");
    }

    // Failure hypothesis: `--file` reads the raw transcript bytes as
    // markdown instead of running the same jq extraction pipeline as
    // `--session`/default resolution, so block-math detection scans the
    // entire JSONL file (every past message, all its code fences) instead
    // of just the last assistant message.
    #[test]
    fn load_markdown_file_source_extracts_via_transcript_pipeline() {
        let path = write_synthetic_transcript("load-markdown");
        let source = Source::File(path.to_string_lossy().into_owned());
        let result = load_markdown(&source);
        let _ = fs::remove_dir_all(path.parent().unwrap());
        assert_eq!(result.unwrap(), "final answer");
    }

    // --- parse_args ---

    #[test]
    fn parse_args_defaults_to_transcript_source_rotate_dpi_130() {
        let args = parse_args(&[]).unwrap();
        assert_eq!(args.source, Source::DefaultTranscript);
        assert_eq!(args.color, ColorMode::Rotate);
        assert_eq!(args.dpi, 130);
        assert!(!args.list);
    }

    #[test]
    fn parse_args_recognizes_stdin_forms() {
        assert_eq!(parse_args(&["-".to_string()]).unwrap().source, Source::Stdin);
        assert_eq!(
            parse_args(&["--stdin".to_string()]).unwrap().source,
            Source::Stdin
        );
    }

    #[test]
    fn parse_args_recognizes_file_and_session_and_list() {
        let args = parse_args(&[
            "--file".to_string(),
            "/tmp/x.md".to_string(),
            "--color".to_string(),
            "gold".to_string(),
            "--dpi".to_string(),
            "200".to_string(),
            "--list".to_string(),
        ])
        .unwrap();
        assert_eq!(args.source, Source::File("/tmp/x.md".to_string()));
        assert_eq!(args.color, ColorMode::Fixed("gold".to_string()));
        assert_eq!(args.dpi, 200);
        assert!(args.list);

        let args = parse_args(&["--session".to_string(), "abc-123".to_string()]).unwrap();
        assert_eq!(args.source, Source::Session("abc-123".to_string()));
    }

    #[test]
    fn parse_args_dry_run_is_an_alias_for_list() {
        assert!(parse_args(&["--dry-run".to_string()]).unwrap().list);
    }

    #[test]
    fn parse_args_rotate_flag_and_color_flag_set_color_mode() {
        assert_eq!(
            parse_args(&["--rotate".to_string()]).unwrap().color,
            ColorMode::Rotate
        );
        assert_eq!(
            parse_args(&["--color".to_string(), "pink".to_string()])
                .unwrap()
                .color,
            ColorMode::Fixed("pink".to_string())
        );
    }

    #[test]
    fn parse_args_rejects_unrecognized_flags() {
        assert!(parse_args(&["--bogus".to_string()]).is_err());
    }

    #[test]
    fn parse_args_rejects_missing_flag_values() {
        assert!(parse_args(&["--file".to_string()]).is_err());
        assert!(parse_args(&["--dpi".to_string()]).is_err());
        assert!(parse_args(&["--dpi".to_string(), "notanumber".to_string()]).is_err());
    }
}
