//! Source format, validation, and HTML output.
//!
//! A source file is a sequence of segments. Each segment opens with a line
//! that is exactly `---`, carries YAML up to the next `---` line, and then a
//! markdown body up to the next `---` line or the end of the file. Because
//! every `---` line is a delimiter, a markdown thematic break inside a body
//! must be written `***`.

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};

use pulldown_cmark::{html, Options, Parser};
use serde::Deserialize;

#[derive(Debug, PartialEq)]
pub struct Segment {
    pub yaml: String,
    pub body: String,
}

pub fn split_segments(text: &str) -> Result<Vec<Segment>, String> {
    let mut lines = text.lines().map(|l| l.strip_suffix('\r').unwrap_or(l));
    match lines.next() {
        Some("---") => {}
        _ => return Err("first line must be `---`".into()),
    }
    let mut segments = Vec::new();
    let mut yaml = String::new();
    let mut body = String::new();
    let mut in_yaml = true;
    for line in lines {
        if line == "---" {
            if in_yaml {
                in_yaml = false;
            } else {
                segments.push(Segment { yaml: std::mem::take(&mut yaml), body: std::mem::take(&mut body) });
                in_yaml = true;
            }
            continue;
        }
        let buf = if in_yaml { &mut yaml } else { &mut body };
        buf.push_str(line);
        buf.push('\n');
    }
    if in_yaml {
        return Err("frontmatter block is not closed with `---` (a thematic break in a body must be `***`)".into());
    }
    segments.push(Segment { yaml, body });
    Ok(segments)
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SiteMeta {
    pub title: String,
    pub description: Option<String>,
    /// Absolute base URL, used for og:url.
    pub url: Option<String>,
    /// Site-relative path of the share image.
    pub image: Option<String>,
    /// Twitter handle without `@`.
    pub twitter: Option<String>,
    #[serde(default = "default_lang")]
    pub lang: String,
    /// Feed author; required once any page is `listed`.
    pub author: Option<String>,
}

fn default_lang() -> String {
    "en".into()
}

#[derive(Debug, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct Card {
    pub title: Option<String>,
    pub image: Option<String>,
    /// Required with `image`; an empty string marks the image decorative.
    pub image_alt: Option<String>,
    /// Source path of a page under the source dir, e.g. `talks/foo.md`.
    pub page: Option<String>,
    /// External URL for the title.
    pub link: Option<String>,
    /// CSS color for the card face.
    pub background: Option<String>,
    /// Drop the content padding (embeds that fill the card).
    #[serde(default)]
    pub flush: bool,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PageMeta {
    pub title: String,
    /// `YYYY-MM-DD`.
    pub date: Option<String>,
    pub venue: Option<String>,
    pub topic: Option<String>,
    pub description: Option<String>,
    /// Directory of an exported slide deck, relative to the page's own URL.
    pub deck: Option<String>,
    /// PNG shown above the title and used as the share image, relative to the
    /// page's own URL. Links to the deck when there is one.
    pub banner: Option<String>,
    /// Put the page in the index's dated list and in the feed. Needs `date`.
    #[serde(default)]
    pub listed: bool,
}

/// One row of the index's dated list and one feed entry.
#[derive(Debug, Clone, PartialEq)]
pub struct Listed {
    pub date: String,
    pub title: String,
    pub url: String,
    pub venue: Option<String>,
    pub description: Option<String>,
}

fn render_list(items: &[Listed]) -> String {
    if items.is_empty() {
        return String::new();
    }
    let mut s = String::from("<ol class=\"dated\">\n");
    for it in items {
        let _ = write!(
            s,
            "<li><time datetime=\"{d}\">{d}</time> <a href=\"{}\">{}</a>",
            escape(&it.url),
            escape(&it.title),
            d = escape(&it.date)
        );
        if let Some(v) = &it.venue {
            let _ = write!(s, " <span class=\"dated__venue\">{}</span>", escape(v));
        }
        s.push_str("</li>\n");
    }
    s.push_str("</ol>\n");
    s
}

/// Atom 1.0 (RFC 4287) for `items`, newest first.
pub fn render_feed(site: &SiteMeta, base: &str, author: &str, items: &[Listed]) -> String {
    let base = base.trim_end_matches('/');
    let updated = items.first().map(|i| i.date.as_str()).unwrap_or("1970-01-01");
    let mut s = String::from("<?xml version=\"1.0\" encoding=\"utf-8\"?>\n<feed xmlns=\"http://www.w3.org/2005/Atom\">\n");
    let _ = writeln!(s, "<title>{}</title>", escape(&site.title));
    let _ = writeln!(s, "<id>{}/</id>", escape(base));
    let _ = writeln!(s, "<link href=\"{}/\"/>", escape(base));
    let _ = writeln!(s, "<link rel=\"self\" href=\"{}/feed.xml\"/>", escape(base));
    let _ = writeln!(s, "<updated>{updated}T00:00:00Z</updated>");
    let _ = writeln!(s, "<author><name>{}</name></author>", escape(author));
    for it in items {
        let url = format!("{base}{}", it.url);
        let _ = writeln!(s, "<entry>\n<title>{}</title>", escape(&it.title));
        let _ = writeln!(s, "<id>{0}</id>\n<link href=\"{0}\"/>", escape(&url));
        let _ = writeln!(s, "<updated>{}T00:00:00Z</updated>", escape(&it.date));
        if let Some(d) = &it.description {
            let _ = writeln!(s, "<summary>{}</summary>", escape(d));
        }
        s.push_str("</entry>\n");
    }
    s.push_str("</feed>\n");
    s
}

/// Width and height from a PNG's IHDR chunk.
pub fn png_size(bytes: &[u8]) -> Result<(u32, u32), String> {
    const SIGNATURE: &[u8] = b"\x89PNG\r\n\x1a\n";
    if bytes.len() < 24 || &bytes[..8] != SIGNATURE || &bytes[12..16] != b"IHDR" {
        return Err("not a PNG".into());
    }
    let be = |b: &[u8]| u32::from_be_bytes([b[0], b[1], b[2], b[3]]);
    Ok((be(&bytes[16..20]), be(&bytes[20..24])))
}

fn parse_yaml<T: for<'de> Deserialize<'de>>(yaml: &str, what: &str) -> Result<T, String> {
    serde_yml::from_str(yaml).map_err(|e| format!("{what}: {e}"))
}

pub fn escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            c => out.push(c),
        }
    }
    out
}

pub fn markdown(src: &str) -> String {
    let opts = Options::ENABLE_TABLES | Options::ENABLE_FOOTNOTES | Options::ENABLE_STRIKETHROUGH;
    let mut out = String::new();
    html::push_html(&mut out, Parser::new_ext(src, opts));
    out
}

const MONTHS: [&str; 12] = [
    "January", "February", "March", "April", "May", "June", "July", "August", "September", "October", "November",
    "December",
];

/// `2026-10-01` to `1 October 2026`.
pub fn long_date(iso: &str) -> Result<String, String> {
    let bad = || format!("date `{iso}` is not YYYY-MM-DD");
    let parts: Vec<&str> = iso.split('-').collect();
    let [y, m, d] = parts.as_slice() else { return Err(bad()) };
    if y.len() != 4 || m.len() != 2 || d.len() != 2 {
        return Err(bad());
    }
    let year: u32 = y.parse().map_err(|_| bad())?;
    let month: usize = m.parse().map_err(|_| bad())?;
    let day: u32 = d.parse().map_err(|_| bad())?;
    if !(1..=12).contains(&month) || !(1..=31).contains(&day) {
        return Err(bad());
    }
    Ok(format!("{day} {} {year}", MONTHS[month - 1]))
}

/// `talks/foo.md` to `talks/foo`.
fn page_stem(rel: &Path) -> PathBuf {
    rel.with_extension("")
}

fn url_for(rel: &Path) -> String {
    let stem = page_stem(rel);
    let parts: Vec<String> = stem.components().map(|c| c.as_os_str().to_string_lossy().into_owned()).collect();
    format!("/{}/", parts.join("/"))
}

fn is_hidden(name: &str) -> bool {
    name.starts_with('_') || name.starts_with('.')
}

/// Every `.md` under `src` except the root `index.md`, as sorted relative paths.
pub fn find_pages(src: &Path) -> Result<Vec<PathBuf>, String> {
    fn walk(root: &Path, dir: &Path, out: &mut Vec<PathBuf>) -> Result<(), String> {
        let entries = fs::read_dir(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
        for entry in entries {
            let entry = entry.map_err(|e| format!("{}: {e}", dir.display()))?;
            let name = entry.file_name().to_string_lossy().into_owned();
            if is_hidden(&name) {
                continue;
            }
            let path = entry.path();
            if path.is_dir() {
                walk(root, &path, out)?;
            } else if path.extension().is_some_and(|e| e == "md") {
                let rel = path.strip_prefix(root).map_err(|e| e.to_string())?.to_path_buf();
                if rel != Path::new("index.md") {
                    out.push(rel);
                }
            }
        }
        Ok(())
    }
    let mut out = Vec::new();
    walk(src, src, &mut out)?;
    out.sort();
    Ok(out)
}

struct Head<'a> {
    site: &'a SiteMeta,
    title: &'a str,
    description: Option<&'a str>,
    path: &'a str,
    /// Site path of the share image; the site's image when `None`.
    image: Option<&'a str>,
    og_type: &'a str,
    extra: &'a str,
}

fn render_head(h: &Head) -> String {
    let mut s = String::new();
    let _ = writeln!(s, "<!DOCTYPE html>\n<html lang=\"{}\">\n<head>", escape(&h.site.lang));
    s.push_str("<meta charset=\"utf-8\">\n<meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\n");
    let _ = writeln!(s, "<title>{}</title>", escape(h.title));
    if let Some(d) = h.description {
        let _ = writeln!(s, "<meta name=\"description\" content=\"{}\">", escape(d));
        let _ = writeln!(s, "<meta property=\"og:description\" content=\"{}\">", escape(d));
    }
    let _ = writeln!(s, "<meta property=\"og:site_name\" content=\"{}\">", escape(&h.site.title));
    let _ = writeln!(s, "<meta property=\"og:title\" content=\"{}\">", escape(h.title));
    let _ = writeln!(s, "<meta property=\"og:type\" content=\"{}\">", h.og_type);
    if let Some(base) = &h.site.url {
        let base = base.trim_end_matches('/');
        let _ = writeln!(s, "<meta property=\"og:url\" content=\"{}{}\">", escape(base), escape(h.path));
        if let Some(img) = h.image.or(h.site.image.as_deref()) {
            let _ = writeln!(s, "<meta property=\"og:image\" content=\"{}{}\">", escape(base), escape(img));
        }
    }
    if let Some(t) = &h.site.twitter {
        let _ = writeln!(s, "<meta name=\"twitter:site\" content=\"@{}\">", escape(t));
    }
    s.push_str(h.extra);
    s.push_str("<link rel=\"stylesheet\" href=\"/css/site.css\">\n</head>\n");
    s
}

fn render_card(card: &Card, body: &str, pages: &BTreeMap<PathBuf, String>) -> Result<String, String> {
    let label = card.title.as_deref().unwrap_or("(untitled card)");
    if card.page.is_some() && card.link.is_some() {
        return Err(format!("card `{label}`: set `page` or `link`, not both"));
    }
    let href = match (&card.page, &card.link) {
        (Some(p), _) => Some(
            pages
                .get(Path::new(p))
                .cloned()
                .ok_or_else(|| format!("card `{label}`: page `{p}` is not a page under the source dir"))?,
        ),
        (None, Some(l)) => Some(l.clone()),
        (None, None) => None,
    };
    let mut s = String::from("<li class=\"cards__item\">\n<article class=\"card\"");
    if let Some(bg) = &card.background {
        let _ = write!(s, " style=\"background-color: {}\"", escape(bg));
    }
    s.push_str(">\n");
    if let Some(img) = &card.image {
        let alt = card
            .image_alt
            .as_deref()
            .ok_or_else(|| format!("card `{label}`: `image` needs `image_alt` (empty string if decorative)"))?;
        let _ = writeln!(s, "<img class=\"card__image\" src=\"{}\" alt=\"{}\" loading=\"lazy\">", escape(img), escape(alt));
    }
    let class = if card.flush { "card__content card__content--flush" } else { "card__content" };
    let _ = writeln!(s, "<div class=\"{class}\">");
    if let Some(title) = &card.title {
        match &href {
            Some(h) => {
                let _ = writeln!(s, "<h2 class=\"card__title\"><a href=\"{}\">{}</a></h2>", escape(h), escape(title));
            }
            None => {
                let _ = writeln!(s, "<h2 class=\"card__title\">{}</h2>", escape(title));
            }
        }
    } else if href.is_some() {
        return Err(format!("card `{label}`: `page` or `link` needs a `title` to carry the link"));
    }
    let _ = write!(s, "<div class=\"card__text\">\n{}</div>\n</div>\n</article>\n</li>\n", markdown(body));
    Ok(s)
}

fn render_index(
    site: &SiteMeta,
    cards: &[(Card, String)],
    pages: &BTreeMap<PathBuf, String>,
    listed: &[Listed],
    head_extra: &str,
) -> Result<String, String> {
    let mut s = render_head(&Head {
        site,
        title: &site.title,
        description: site.description.as_deref(),
        path: "/",
        image: None,
        og_type: "website",
        extra: head_extra,
    });
    let _ = writeln!(s, "<body>\n<h1 class=\"visually-hidden\">{}</h1>\n<main>", escape(&site.title));
    s.push_str("<ul class=\"cards\">\n");
    for (card, body) in cards {
        s.push_str(&render_card(card, body, pages)?);
    }
    s.push_str("</ul>\n");
    s.push_str(&render_list(listed));
    s.push_str("</main>\n</body>\n</html>\n");
    Ok(s)
}

fn render_page(
    site: &SiteMeta,
    meta: &PageMeta,
    body: &str,
    url: &str,
    head_extra: &str,
    deck_help: Option<&str>,
    banner_size: Option<(u32, u32)>,
) -> Result<String, String> {
    let banner_url = meta.banner.as_ref().map(|b| format!("{url}{b}"));
    let mut s = render_head(&Head {
        site,
        title: &meta.title,
        description: meta.description.as_deref(),
        path: url,
        image: banner_url.as_deref(),
        og_type: "article",
        extra: head_extra,
    });
    let _ = writeln!(s, "<body class=\"page\">\n<main class=\"article\">");
    let _ = writeln!(
        s,
        "<nav class=\"article__nav\"><a href=\"/\"><span aria-hidden=\"true\">←</span> {}</a></nav>",
        escape(&site.title)
    );
    if let (Some(banner), Some((w, h))) = (&meta.banner, banner_size) {
        let img = format!(
            "<img src=\"{}\" width=\"{w}\" height=\"{h}\" alt=\"{}\">",
            escape(banner),
            escape(&meta.title)
        );
        match &meta.deck {
            Some(d) => {
                let _ = writeln!(s, "<a class=\"talk-banner\" href=\"{}\">{img}</a>", escape(d));
            }
            None => {
                let _ = writeln!(s, "<p class=\"talk-banner\">{img}</p>");
            }
        }
    }
    let _ = writeln!(s, "<header>\n<h1>{}</h1>", escape(&meta.title));
    let mut line = Vec::new();
    if let Some(d) = &meta.date {
        line.push(format!("<time datetime=\"{}\">{}</time>", escape(d), long_date(d)?));
    }
    for v in [&meta.venue, &meta.topic].into_iter().flatten() {
        line.push(escape(v));
    }
    if !line.is_empty() {
        let _ = writeln!(s, "<p class=\"article__meta\">{}</p>", line.join(" · "));
    }
    s.push_str("</header>\n");
    if let Some(deck) = &meta.deck {
        let help = deck_help.ok_or("`deck` is set but _partials/deck-help.md is missing")?;
        let _ = write!(
            s,
            "<section class=\"deck\" aria-label=\"Slides\">\n\
             <div class=\"deck__frame\"><iframe src=\"{}\" title=\"Slides: {}\" allow=\"fullscreen\" loading=\"lazy\"></iframe></div>\n\
             <div class=\"deck__help\">\n{}</div>\n</section>\n",
            escape(deck),
            escape(&meta.title),
            markdown(help)
        );
    }
    s.push_str(&markdown(body));
    s.push_str("</main>\n</body>\n</html>\n");
    Ok(s)
}

fn read(path: &Path) -> Result<String, String> {
    fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))
}

fn read_optional(path: &Path) -> Result<Option<String>, String> {
    match fs::read_to_string(path) {
        Ok(s) => Ok(Some(s)),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(format!("{}: {e}", path.display())),
    }
}

fn write(path: &Path, contents: &str) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("{}: {e}", parent.display()))?;
    }
    fs::write(path, contents).map_err(|e| format!("{}: {e}", path.display()))
}

/// Build everything; returns the files written. Nothing is written unless
/// every source file parses and every reference resolves.
pub fn build(src: &Path, out: &Path) -> Result<Vec<PathBuf>, String> {
    let index_path = src.join("index.md");
    let segments = split_segments(&read(&index_path)?).map_err(|e| format!("{}: {e}", index_path.display()))?;
    let (first, rest) = segments.split_first().ok_or("index.md is empty")?;
    if !first.body.trim().is_empty() {
        return Err("index.md: put card text after a card's frontmatter, not after the site frontmatter".into());
    }
    let site: SiteMeta = parse_yaml(&first.yaml, "index.md site frontmatter")?;
    let mut cards = Vec::new();
    for (i, seg) in rest.iter().enumerate() {
        let card: Card = parse_yaml(&seg.yaml, &format!("index.md card {}", i + 1))?;
        cards.push((card, seg.body.clone()));
    }

    let partials = src.join("_partials");
    let mut head_extra = read_optional(&partials.join("head.html"))?.unwrap_or_default();
    let deck_help = read_optional(&partials.join("deck-help.md"))?;

    let page_paths = find_pages(src)?;
    let urls: BTreeMap<PathBuf, String> = page_paths.iter().map(|p| (p.clone(), url_for(p))).collect();

    let mut pages = Vec::new();
    let mut listed = Vec::new();
    for rel in &page_paths {
        let label = rel.display().to_string();
        let segs = split_segments(&read(&src.join(rel))?).map_err(|e| format!("{label}: {e}"))?;
        let [seg] = segs.as_slice() else {
            return Err(format!("{label}: a page has exactly one frontmatter block; use `***` for a thematic break"));
        };
        let meta: PageMeta = parse_yaml(&seg.yaml, &label)?;
        if meta.listed {
            let date = meta.date.clone().ok_or_else(|| format!("{label}: `listed` needs `date`"))?;
            long_date(&date).map_err(|e| format!("{label}: {e}"))?;
            listed.push(Listed {
                date,
                title: meta.title.clone(),
                url: urls[rel].clone(),
                venue: meta.venue.clone(),
                description: meta.description.clone(),
            });
        }
        pages.push((rel, label, meta, seg.body.clone()));
    }
    // Newest first; same-day pages by title.
    listed.sort_by(|a, b| b.date.cmp(&a.date).then_with(|| a.title.cmp(&b.title)));

    let mut outputs = Vec::new();
    if !listed.is_empty() {
        let base = site.url.as_deref().ok_or("index.md: listed pages need the site `url` for the feed")?;
        let author = site.author.as_deref().ok_or("index.md: listed pages need the site `author` for the feed")?;
        outputs.push((out.join("feed.xml"), render_feed(&site, base, author, &listed)));
        let _ = writeln!(
            head_extra,
            "<link rel=\"alternate\" type=\"application/atom+xml\" href=\"/feed.xml\" title=\"{}\">",
            escape(&site.title)
        );
    }
    outputs.push((out.join("index.html"), render_index(&site, &cards, &urls, &listed, &head_extra)?));
    for (rel, label, meta, body) in &pages {
        let dir = out.join(page_stem(rel));
        if let Some(deck) = &meta.deck {
            let deck_index = dir.join(deck).join("index.html");
            if !deck_index.is_file() {
                return Err(format!("{label}: deck `{deck}` has no {}; export the deck first", deck_index.display()));
            }
        }
        let banner_size = match &meta.banner {
            Some(b) => {
                let path = dir.join(b);
                let bytes = fs::read(&path).map_err(|e| format!("{label}: banner {}: {e}", path.display()))?;
                Some(png_size(&bytes).map_err(|e| format!("{label}: banner {}: {e}", path.display()))?)
            }
            None => None,
        };
        let html = render_page(&site, meta, body, &urls[*rel], &head_extra, deck_help.as_deref(), banner_size)
            .map_err(|e| format!("{label}: {e}"))?;
        outputs.push((dir.join("index.html"), html));
    }

    let mut written = Vec::new();
    for (path, html) in outputs {
        write(&path, &html)?;
        written.push(path);
    }
    Ok(written)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tempdir(name: &str) -> PathBuf {
        let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("target/test-tmp").join(name);
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn splits_alternating_frontmatter_and_bodies() {
        let segs = split_segments("---\na: 1\n---\nbody one\n---\nb: 2\n---\nbody two\n").unwrap();
        assert_eq!(
            segs,
            vec![
                Segment { yaml: "a: 1\n".into(), body: "body one\n".into() },
                Segment { yaml: "b: 2\n".into(), body: "body two\n".into() },
            ]
        );
    }

    #[test]
    fn rejects_file_not_starting_with_delimiter() {
        assert!(split_segments("a: 1\n---\n").is_err());
    }

    #[test]
    fn rejects_unclosed_frontmatter() {
        assert!(split_segments("---\na: 1\n---\nbody\n---\nb: 2\n").is_err());
    }

    #[test]
    fn accepts_crlf_delimiters() {
        let segs = split_segments("---\r\na: 1\r\n---\r\nx\r\n").unwrap();
        assert_eq!(segs.len(), 1);
        assert_eq!(segs[0].body, "x\n");
    }

    #[test]
    fn rejects_unknown_card_field() {
        assert!(parse_yaml::<Card>("titel: x\n", "card").is_err());
    }

    #[test]
    fn escapes_attribute_and_text_metacharacters() {
        assert_eq!(escape("<a href=\"x\">'&'</a>"), "&lt;a href=&quot;x&quot;&gt;&#39;&amp;&#39;&lt;/a&gt;");
    }

    #[test]
    fn formats_long_date_and_rejects_malformed() {
        assert_eq!(long_date("2026-10-01").unwrap(), "1 October 2026");
        assert!(long_date("2026-13-01").is_err());
        assert!(long_date("2026-1-01").is_err());
        assert!(long_date("Oct 1").is_err());
    }

    #[test]
    fn url_for_nested_page_is_directory_url() {
        assert_eq!(url_for(Path::new("talks/foo.md")), "/talks/foo/");
    }

    #[test]
    fn image_without_alt_fails() {
        let card = Card { image: Some("/a.jpg".into()), ..Default::default() };
        assert!(render_card(&card, "", &BTreeMap::new()).is_err());
    }

    #[test]
    fn empty_alt_marks_image_decorative() {
        let card = Card { image: Some("/a.jpg".into()), image_alt: Some(String::new()), ..Default::default() };
        assert!(render_card(&card, "", &BTreeMap::new()).unwrap().contains("alt=\"\""));
    }

    #[test]
    fn card_page_must_exist() {
        let card = Card { title: Some("t".into()), page: Some("talks/missing.md".into()), ..Default::default() };
        let err = render_card(&card, "", &BTreeMap::new()).unwrap_err();
        assert!(err.contains("talks/missing.md"), "{err}");
    }

    #[test]
    fn card_page_links_title_to_built_url() {
        let mut pages = BTreeMap::new();
        pages.insert(PathBuf::from("talks/x.md"), "/talks/x/".to_string());
        let card = Card { title: Some("T".into()), page: Some("talks/x.md".into()), ..Default::default() };
        assert!(render_card(&card, "", &pages).unwrap().contains("<a href=\"/talks/x/\">T</a>"));
    }

    #[test]
    fn page_and_link_together_fail() {
        let card = Card {
            title: Some("t".into()),
            page: Some("a.md".into()),
            link: Some("https://example.com".into()),
            ..Default::default()
        };
        assert!(render_card(&card, "", &BTreeMap::new()).is_err());
    }

    #[test]
    fn link_without_title_fails() {
        let card = Card { link: Some("https://example.com".into()), ..Default::default() };
        assert!(render_card(&card, "", &BTreeMap::new()).is_err());
    }

    #[test]
    fn raw_html_in_body_passes_through() {
        assert!(markdown("<canvas id=\"c\"></canvas>\n").contains("<canvas id=\"c\"></canvas>"));
    }

    fn write_site(src: &Path, page: &str) {
        write(&src.join("index.md"), "---\ntitle: Site\n---\n---\ntitle: Card\npage: talks/x.md\n---\nhi\n").unwrap();
        write(&src.join("talks/x.md"), page).unwrap();
    }

    #[test]
    fn builds_index_and_unlisted_pages() {
        let root = tempdir("builds_index_and_unlisted_pages");
        let (src, out) = (root.join("src"), root.join("out"));
        write_site(&src, "---\ntitle: X\ndate: 2026-06-01\n---\nbody\n");
        write(&src.join("notes/hidden.md"), "---\ntitle: Hidden\n---\nonly by link\n").unwrap();
        write(&src.join("_partials/ignored.md"), "not a page").unwrap();
        let written = build(&src, &out).unwrap();
        assert_eq!(
            written,
            vec![out.join("index.html"), out.join("notes/hidden/index.html"), out.join("talks/x/index.html")]
        );
        let page = fs::read_to_string(out.join("talks/x/index.html")).unwrap();
        assert!(page.contains("<time datetime=\"2026-06-01\">1 June 2026</time>"), "{page}");
        let index = fs::read_to_string(out.join("index.html")).unwrap();
        assert!(!index.contains("/notes/hidden/"), "unlisted page leaked into the index");
    }

    #[test]
    fn deck_without_export_fails_and_writes_nothing() {
        let root = tempdir("deck_without_export_fails_and_writes_nothing");
        let (src, out) = (root.join("src"), root.join("out"));
        write_site(&src, "---\ntitle: X\ndeck: deck/\n---\nbody\n");
        write(&src.join("_partials/deck-help.md"), "keys\n").unwrap();
        let err = build(&src, &out).unwrap_err();
        assert!(err.contains("export the deck first"), "{err}");
        assert!(!out.join("index.html").exists());
    }

    #[test]
    fn deck_without_help_partial_fails() {
        let root = tempdir("deck_without_help_partial_fails");
        let (src, out) = (root.join("src"), root.join("out"));
        write_site(&src, "---\ntitle: X\ndeck: deck/\n---\nbody\n");
        write(&out.join("talks/x/deck/index.html"), "").unwrap();
        let err = build(&src, &out).unwrap_err();
        assert!(err.contains("deck-help.md"), "{err}");
    }

    #[test]
    fn deck_renders_frame_and_help() {
        let root = tempdir("deck_renders_frame_and_help");
        let (src, out) = (root.join("src"), root.join("out"));
        write_site(&src, "---\ntitle: X\ndeck: deck/\n---\nbody\n");
        write(&src.join("_partials/deck-help.md"), "Press `?`.\n").unwrap();
        write(&out.join("talks/x/deck/index.html"), "").unwrap();
        build(&src, &out).unwrap();
        let page = fs::read_to_string(out.join("talks/x/index.html")).unwrap();
        assert!(page.contains("<iframe src=\"deck/\""), "{page}");
        assert!(page.contains("Press <code>?</code>."), "{page}");
    }

    fn png_header(w: u32, h: u32) -> Vec<u8> {
        let mut b = b"\x89PNG\r\n\x1a\n\0\0\0\x0dIHDR".to_vec();
        b.extend_from_slice(&w.to_be_bytes());
        b.extend_from_slice(&h.to_be_bytes());
        b
    }

    #[test]
    fn png_size_reads_ihdr_and_rejects_other_files() {
        assert_eq!(png_size(&png_header(1280, 800)).unwrap(), (1280, 800));
        assert!(png_size(b"\xff\xd8\xff\xe0 not a png at all....").is_err());
    }

    #[test]
    fn banner_links_to_deck_sets_share_image_and_deck_precedes_text() {
        let root = tempdir("banner_links_to_deck");
        let (src, out) = (root.join("src"), root.join("out"));
        write(&src.join("index.md"), "---\ntitle: Site\nurl: https://example.org\n---\n").unwrap();
        write(&src.join("talks/x.md"), "---\ntitle: X\ndeck: deck/\nbanner: banner.png\n---\nabstract\n").unwrap();
        write(&src.join("_partials/deck-help.md"), "keys\n").unwrap();
        write(&out.join("talks/x/deck/index.html"), "").unwrap();
        fs::write(out.join("talks/x/banner.png"), png_header(1280, 800)).unwrap();
        build(&src, &out).unwrap();
        let page = fs::read_to_string(out.join("talks/x/index.html")).unwrap();
        assert!(
            page.contains("<a class=\"talk-banner\" href=\"deck/\"><img src=\"banner.png\" width=\"1280\" height=\"800\" alt=\"X\"></a>"),
            "{page}"
        );
        assert!(page.contains("og:image\" content=\"https://example.org/talks/x/banner.png\""), "{page}");
        assert!(page.find("class=\"deck\"").unwrap() < page.find("<p>abstract</p>").unwrap(), "{page}");
    }

    fn listed_site(root: &Path, site_extra: &str) -> (PathBuf, PathBuf) {
        let (src, out) = (root.join("src"), root.join("out"));
        write(&src.join("index.md"), &format!("---\ntitle: Site\n{site_extra}---\n")).unwrap();
        write(&src.join("talks/old.md"), "---\ntitle: Old\ndate: 2026-06-01\nvenue: V1\nlisted: true\n---\n").unwrap();
        write(&src.join("talks/new.md"), "---\ntitle: New & Shiny\ndate: 2026-10-01\nlisted: true\ndescription: d\n---\n").unwrap();
        write(&src.join("talks/hidden.md"), "---\ntitle: Hidden\ndate: 2026-12-01\n---\n").unwrap();
        (src, out)
    }

    #[test]
    fn listed_pages_form_a_dated_list_newest_first_below_the_cards() {
        let root = tempdir("listed_dated_list");
        let (src, out) = listed_site(&root, "url: https://example.org\nauthor: A\n");
        build(&src, &out).unwrap();
        let index = fs::read_to_string(out.join("index.html")).unwrap();
        assert!(index.find("class=\"cards\"").unwrap() < index.find("class=\"dated\"").unwrap(), "{index}");
        let (new, old) = (index.find("New &amp; Shiny").unwrap(), index.find(">Old<").unwrap());
        assert!(new < old, "{index}");
        assert!(index.contains("<time datetime=\"2026-06-01\">2026-06-01</time> <a href=\"/talks/old/\">Old</a> <span class=\"dated__venue\">V1</span>"), "{index}");
        assert!(!index.contains("Hidden"), "unlisted page leaked into the list");
        assert!(index.contains("rel=\"alternate\" type=\"application/atom+xml\" href=\"/feed.xml\""), "{index}");
    }

    #[test]
    fn feed_has_absolute_entry_links_and_newest_updated() {
        let root = tempdir("feed_entries");
        let (src, out) = listed_site(&root, "url: https://example.org/\nauthor: A\n");
        build(&src, &out).unwrap();
        let feed = fs::read_to_string(out.join("feed.xml")).unwrap();
        assert!(feed.contains("<updated>2026-10-01T00:00:00Z</updated>\n<author>"), "{feed}");
        assert!(feed.contains("<id>https://example.org/talks/new/</id>"), "{feed}");
        assert!(feed.contains("<title>New &amp; Shiny</title>"), "{feed}");
        assert!(feed.contains("<summary>d</summary>"), "{feed}");
        assert!(!feed.contains("Hidden"), "{feed}");
        assert!(feed.find("/talks/new/").unwrap() < feed.find("/talks/old/").unwrap());
    }

    #[test]
    fn listed_without_url_or_author_fails() {
        let root = tempdir("listed_needs_url");
        let (src, out) = listed_site(&root, "author: A\n");
        assert!(build(&src, &out).unwrap_err().contains("url"));
        let root = tempdir("listed_needs_author");
        let (src, out) = listed_site(&root, "url: https://example.org\n");
        assert!(build(&src, &out).unwrap_err().contains("author"));
    }

    #[test]
    fn listed_without_date_fails() {
        let root = tempdir("listed_needs_date");
        let (src, out) = (root.join("src"), root.join("out"));
        write(&src.join("index.md"), "---\ntitle: Site\nurl: https://e.org\nauthor: A\n---\n").unwrap();
        write(&src.join("p.md"), "---\ntitle: P\nlisted: true\n---\n").unwrap();
        assert!(build(&src, &out).unwrap_err().contains("`listed` needs `date`"));
    }

    #[test]
    fn no_listed_pages_means_no_feed() {
        let root = tempdir("no_feed");
        let (src, out) = (root.join("src"), root.join("out"));
        write(&src.join("index.md"), "---\ntitle: Site\n---\n").unwrap();
        build(&src, &out).unwrap();
        assert!(!out.join("feed.xml").exists());
        assert!(!fs::read_to_string(out.join("index.html")).unwrap().contains("feed.xml"));
    }

    #[test]
    fn missing_banner_fails() {
        let root = tempdir("missing_banner_fails");
        let (src, out) = (root.join("src"), root.join("out"));
        write(&src.join("index.md"), "---\ntitle: Site\n---\n").unwrap();
        write(&src.join("talks/x.md"), "---\ntitle: X\nbanner: banner.png\n---\n").unwrap();
        let err = build(&src, &out).unwrap_err();
        assert!(err.contains("banner"), "{err}");
    }

    #[test]
    fn page_with_second_segment_fails() {
        let root = tempdir("page_with_second_segment_fails");
        let (src, out) = (root.join("src"), root.join("out"));
        write_site(&src, "---\ntitle: X\n---\nabove\n---\nbelow\n");
        let err = build(&src, &out).unwrap_err();
        assert!(err.contains("***"), "{err}");
    }
}
