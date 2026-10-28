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
    /// What the share image shows. Required with `image`.
    pub image_alt: Option<String>,
    /// Twitter handle without `@`.
    pub twitter: Option<String>,
    #[serde(default = "default_lang")]
    pub lang: String,
    /// Feed author; required once any page sets `feed`.
    pub author: Option<String>,
    /// Heading of the header at the top of every page. The header is drawn
    /// only when this is set.
    pub name: Option<String>,
    /// Markdown under the name. Needs `name`.
    pub bio: Option<String>,
    /// Profiles elsewhere, in order, shown as one row of words. Needs `name`.
    #[serde(default)]
    pub links: Vec<ProfileLink>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProfileLink {
    /// Link text, or with `icon` the accessible name and tooltip.
    pub label: String,
    pub url: String,
    /// Site path of an SVG drawn in place of the label. It is inlined so it
    /// takes the text color.
    pub icon: Option<String>,
    /// The `icon` file's contents, read by [`build`].
    #[serde(skip)]
    pub svg: Option<String>,
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
    /// Share image (`og:image`), relative to the page's own URL.
    pub image: Option<String>,
    /// What the share image shows. Required with `image`.
    pub image_alt: Option<String>,
    /// Put the page in `feed.xml`. Needs `date`.
    #[serde(default)]
    pub feed: bool,
}

/// One feed entry.
#[derive(Debug, Clone, PartialEq)]
pub struct FeedEntry {
    pub date: String,
    pub title: String,
    pub url: String,
    pub description: Option<String>,
}

/// Atom 1.0 (RFC 4287) for `items`, newest first.
pub fn render_feed(site: &SiteMeta, base: &str, author: &str, items: &[FeedEntry]) -> String {
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

/// A link-preview image, measured from its file.
#[derive(Debug, Clone, PartialEq)]
pub struct ShareImage {
    /// Site path, e.g. `/talks/x/deck/share.png`.
    pub path: String,
    pub alt: String,
    pub width: usize,
    pub height: usize,
}

/// Measure `file`, which is served at site path `path`.
fn share_image(file: &Path, path: String, alt: &str) -> Result<ShareImage, String> {
    if !file.is_file() {
        return Err(format!("image {} does not exist", file.display()));
    }
    let size = imagesize::size(file).map_err(|e| format!("image {}: {e}", file.display()))?;
    Ok(ShareImage { path, alt: alt.to_string(), width: size.width, height: size.height })
}

struct Head<'a> {
    site: &'a SiteMeta,
    title: &'a str,
    description: Option<&'a str>,
    path: &'a str,
    image: Option<&'a ShareImage>,
    og_type: &'a str,
    /// `YYYY-MM-DD`, for `article:published_time`.
    published: Option<&'a str>,
    /// Label and value pairs Slack shows under the description.
    labels: &'a [(&'a str, String)],
    extra: &'a str,
}

/// Open Graph and Twitter Card tags, which Facebook, LinkedIn, X, Slack,
/// Discord, iMessage, and Mastodon read for link previews. Image URLs must
/// be absolute, so the image tags need the site `url`.
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
    if let Some(d) = h.published {
        let _ = writeln!(s, "<meta property=\"article:published_time\" content=\"{}\">", escape(d));
    }
    let mut large = false;
    if let Some(base) = &h.site.url {
        let base = base.trim_end_matches('/');
        let _ = writeln!(s, "<link rel=\"canonical\" href=\"{}{}\">", escape(base), escape(h.path));
        let _ = writeln!(s, "<meta property=\"og:url\" content=\"{}{}\">", escape(base), escape(h.path));
        if let Some(img) = h.image {
            let _ = writeln!(s, "<meta property=\"og:image\" content=\"{}{}\">", escape(base), escape(&img.path));
            let _ = writeln!(s, "<meta property=\"og:image:width\" content=\"{}\">", img.width);
            let _ = writeln!(s, "<meta property=\"og:image:height\" content=\"{}\">", img.height);
            let _ = writeln!(s, "<meta property=\"og:image:alt\" content=\"{}\">", escape(&img.alt));
            let _ = writeln!(s, "<meta name=\"twitter:image:alt\" content=\"{}\">", escape(&img.alt));
            large = true;
        }
    }
    let card = if large { "summary_large_image" } else { "summary" };
    let _ = writeln!(s, "<meta name=\"twitter:card\" content=\"{card}\">");
    if let Some(t) = &h.site.twitter {
        let _ = writeln!(s, "<meta name=\"twitter:site\" content=\"@{}\">", escape(t));
        let _ = writeln!(s, "<meta name=\"twitter:creator\" content=\"@{}\">", escape(t));
    }
    for (i, (label, data)) in h.labels.iter().enumerate() {
        let n = i + 1;
        let _ = writeln!(s, "<meta name=\"twitter:label{n}\" content=\"{}\">", escape(label));
        let _ = writeln!(s, "<meta name=\"twitter:data{n}\" content=\"{}\">", escape(data));
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
    // A card for a page links as a whole: the stylesheet stretches the title
    // link over the card.
    let mut s = String::from(if card.page.is_some() {
        "<li class=\"cards__item\">\n<article class=\"card card--page\""
    } else {
        "<li class=\"cards__item\">\n<article class=\"card\""
    });
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
    head_extra: &str,
    image: Option<&ShareImage>,
) -> Result<String, String> {
    let mut s = render_head(&Head {
        site,
        title: &site.title,
        description: site.description.as_deref(),
        path: "/",
        image,
        og_type: "website",
        published: None,
        labels: &[],
        extra: head_extra,
    });
    match render_site_header(site, true) {
        Some(header) => {
            let _ = write!(s, "<body>\n{header}<main>\n");
        }
        None => {
            let _ = writeln!(s, "<body>\n<h1 class=\"visually-hidden\">{}</h1>\n<main>", escape(&site.title));
        }
    }
    s.push_str("<ul class=\"cards\">\n");
    for (card, body) in cards {
        s.push_str(&render_card(card, body, pages)?);
    }
    s.push_str("</ul>\n</main>\n</body>\n</html>\n");
    Ok(s)
}

/// The name, bio, and profile links at the top of every page, or `None` when
/// the site has no `name`. The name is the index's `h1`; a page has its own
/// `h1`, so there the name is a paragraph.
fn render_site_header(site: &SiteMeta, is_index: bool) -> Option<String> {
    let name = site.name.as_ref()?;
    let tag = if is_index { "h1" } else { "p" };
    let mut s = String::from("<header class=\"site-header\">\n");
    let _ = writeln!(s, "<{tag} class=\"site-header__name\"><a href=\"/\">{}</a></{tag}>", escape(name));
    if let Some(bio) = &site.bio {
        let _ = write!(s, "<div class=\"site-header__bio\">\n{}</div>\n", markdown(bio));
    }
    if !site.links.is_empty() {
        s.push_str("<ul class=\"site-header__links\">\n");
        for link in &site.links {
            let (url, label) = (escape(&link.url), escape(&link.label));
            match &link.svg {
                Some(svg) => {
                    let _ = writeln!(
                        s,
                        "<li><a rel=\"me\" href=\"{url}\" aria-label=\"{label}\" title=\"{label}\"><span class=\"site-header__icon\" aria-hidden=\"true\">{}</span></a></li>",
                        svg.trim()
                    );
                }
                None => {
                    let _ = writeln!(s, "<li><a rel=\"me\" href=\"{url}\">{label}</a></li>");
                }
            }
        }
        s.push_str("</ul>\n");
    }
    s.push_str("</header>\n");
    Some(s)
}

fn render_page(
    site: &SiteMeta,
    meta: &PageMeta,
    body: &str,
    url: &str,
    head_extra: &str,
    deck_help: Option<&str>,
    image: Option<&ShareImage>,
) -> Result<String, String> {
    let mut labels = Vec::new();
    if let Some(v) = &meta.venue {
        labels.push(("Venue", v.clone()));
    }
    if let Some(d) = &meta.date {
        labels.push(("Date", long_date(d)?));
    }
    let mut s = render_head(&Head {
        site,
        title: &meta.title,
        description: meta.description.as_deref(),
        path: url,
        image,
        og_type: "article",
        published: meta.date.as_deref(),
        labels: &labels,
        extra: head_extra,
    });
    s.push_str("<body class=\"page\">\n");
    s.push_str(&render_site_header(site, false).unwrap_or_default());
    s.push_str("<main class=\"article\">\n");
    let _ = writeln!(
        s,
        "<nav class=\"article__nav\"><a href=\"/\"><span aria-hidden=\"true\">←</span> {}</a></nav>",
        escape(&site.title)
    );
    // The page's one picture is the share image, above the title; with a
    // deck it links to the deck, which opens as its own page.
    let help = match &meta.deck {
        Some(deck) => {
            let help = deck_help.ok_or("`deck` is set but _partials/deck-help.md is missing")?;
            let image = meta.image.as_ref().ok_or("`deck` needs `image` for the picture that opens it")?;
            let alt = meta.image_alt.as_deref().ok_or("`image` needs `image_alt`")?;
            let _ = writeln!(
                s,
                "<a class=\"deck__banner\" href=\"{}\"><img src=\"{}\" alt=\"{}. Opens the slides.\"></a>",
                escape(deck),
                escape(image),
                escape(alt.trim_end_matches('.'))
            );
            Some(help)
        }
        None => None,
    };
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
    if let Some(help) = help {
        let _ = write!(s, "<div class=\"deck__help\">\n{}</div>\n", markdown(help));
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
/// What a build wrote, and the site `url` its pages use for themselves.
#[derive(Debug)]
pub struct Built {
    pub written: Vec<PathBuf>,
    pub url: Option<String>,
}

pub fn build(src: &Path, out: &Path) -> Result<Built, String> {
    let index_path = src.join("index.md");
    let segments = split_segments(&read(&index_path)?).map_err(|e| format!("{}: {e}", index_path.display()))?;
    let (first, rest) = segments.split_first().ok_or("index.md is empty")?;
    if !first.body.trim().is_empty() {
        return Err("index.md: put card text after a card's frontmatter, not after the site frontmatter".into());
    }
    let mut site: SiteMeta = parse_yaml(&first.yaml, "index.md site frontmatter")?;
    if site.name.is_none() && (site.bio.is_some() || !site.links.is_empty()) {
        return Err("index.md: `bio` and `links` go in the header, which needs `name`".into());
    }
    for link in &mut site.links {
        if let Some(icon) = &link.icon {
            let svg = read(&out.join(icon.trim_start_matches('/')))
                .map_err(|e| format!("index.md: icon for `{}`: {e}", link.label))?;
            let is_svg = tl::parse(&svg, tl::ParserOptions::default())
                .is_ok_and(|dom| dom.nodes().iter().filter_map(|n| n.as_tag()).any(|t| t.name() == "svg"));
            if !is_svg {
                return Err(format!("index.md: icon `{icon}` for `{}` has no <svg> element", link.label));
            }
            link.svg = Some(svg);
        }
    }
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
    let mut feed = Vec::new();
    for rel in &page_paths {
        let label = rel.display().to_string();
        let segs = split_segments(&read(&src.join(rel))?).map_err(|e| format!("{label}: {e}"))?;
        let [seg] = segs.as_slice() else {
            return Err(format!("{label}: a page has exactly one frontmatter block; use `***` for a thematic break"));
        };
        let meta: PageMeta = parse_yaml(&seg.yaml, &label)?;
        if meta.feed {
            let date = meta.date.clone().ok_or_else(|| format!("{label}: `feed` needs `date`"))?;
            long_date(&date).map_err(|e| format!("{label}: {e}"))?;
            feed.push(FeedEntry {
                date,
                title: meta.title.clone(),
                url: urls[rel].clone(),
                description: meta.description.clone(),
            });
        }
        pages.push((rel, label, meta, seg.body.clone()));
    }
    // Newest first; same-day pages by title.
    feed.sort_by(|a, b| b.date.cmp(&a.date).then_with(|| a.title.cmp(&b.title)));

    let mut outputs = Vec::new();
    if !feed.is_empty() {
        let base = site.url.as_deref().ok_or("index.md: `feed` pages need the site `url`")?;
        let author = site.author.as_deref().ok_or("index.md: `feed` pages need the site `author`")?;
        outputs.push((out.join("feed.xml"), render_feed(&site, base, author, &feed)));
        let _ = writeln!(
            head_extra,
            "<link rel=\"alternate\" type=\"application/atom+xml\" href=\"/feed.xml\" title=\"{}\">",
            escape(&site.title)
        );
    }
    let site_image = match &site.image {
        Some(image) => {
            let alt = site.image_alt.as_deref().ok_or("index.md: the site `image` needs `image_alt`")?;
            let file = out.join(image.trim_start_matches('/'));
            Some(share_image(&file, image.clone(), alt).map_err(|e| format!("index.md: {e}"))?)
        }
        None => None,
    };
    outputs.push((out.join("index.html"), render_index(&site, &cards, &urls, &head_extra, site_image.as_ref())?));
    for (rel, label, meta, body) in &pages {
        let dir = out.join(page_stem(rel));
        if let Some(deck) = &meta.deck {
            let deck_index = dir.join(deck).join("index.html");
            if !deck_index.is_file() {
                return Err(format!("{label}: deck `{deck}` has no {}; export the deck first", deck_index.display()));
            }
        }
        let page_image = match &meta.image {
            Some(image) => {
                let alt = meta.image_alt.as_deref().ok_or_else(|| format!("{label}: `image` needs `image_alt`"))?;
                let path = format!("{}{image}", urls[*rel]);
                Some(share_image(&dir.join(image), path, alt).map_err(|e| format!("{label}: {e}"))?)
            }
            None => None,
        };
        let image = page_image.as_ref().or(site_image.as_ref());
        let html = render_page(&site, meta, body, &urls[*rel], &head_extra, deck_help.as_deref(), image)
            .map_err(|e| format!("{label}: {e}"))?;
        outputs.push((dir.join("index.html"), html));
    }

    let mut written = Vec::new();
    for (path, html) in outputs {
        write(&path, &html)?;
        written.push(path);
    }
    Ok(Built { written, url: site.url })
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
    fn only_page_cards_link_as_a_whole() {
        let mut pages = BTreeMap::new();
        pages.insert(PathBuf::from("talks/x.md"), "/talks/x/".to_string());
        let page = Card { title: Some("T".into()), page: Some("talks/x.md".into()), ..Default::default() };
        assert!(render_card(&page, "", &pages).unwrap().contains("<article class=\"card card--page\">"));
        let link = Card { title: Some("T".into()), link: Some("https://example.com".into()), ..Default::default() };
        assert!(render_card(&link, "", &pages).unwrap().contains("<article class=\"card\">"));
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
        let written = build(&src, &out).unwrap().written;
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
        write_site(&src, "---\ntitle: X\ndeck: deck/\nimage: deck/thumbnail.png\nimage_alt: Title slide of X\n---\nbody\n");
        write(&src.join("_partials/deck-help.md"), "keys\n").unwrap();
        let err = build(&src, &out).unwrap_err();
        assert!(err.contains("export the deck first"), "{err}");
        assert!(!out.join("index.html").exists());
    }

    #[test]
    fn deck_without_help_partial_fails() {
        let root = tempdir("deck_without_help_partial_fails");
        let (src, out) = (root.join("src"), root.join("out"));
        write_site(&src, "---\ntitle: X\ndeck: deck/\nimage: deck/thumbnail.png\nimage_alt: Title slide of X\n---\nbody\n");
        write(&out.join("talks/x/deck/index.html"), "").unwrap();
        put_png(&out.join("talks/x/deck/thumbnail.png"), 1200, 630);
        let err = build(&src, &out).unwrap_err();
        assert!(err.contains("deck-help.md"), "{err}");
    }

    #[test]
    fn deck_without_image_fails() {
        // Fails if a deck page builds with nothing to click to open the deck.
        let root = tempdir("deck_without_image_fails");
        let (src, out) = (root.join("src"), root.join("out"));
        write_site(&src, "---\ntitle: X\ndeck: deck/\n---\nbody\n");
        write(&src.join("_partials/deck-help.md"), "keys\n").unwrap();
        write(&out.join("talks/x/deck/index.html"), "").unwrap();
        let err = build(&src, &out).unwrap_err();
        assert!(err.contains("`deck` needs `image`"), "{err}");
    }

    #[test]
    fn deck_renders_banner_link_and_help() {
        let root = tempdir("deck_renders_banner_link_and_help");
        let (src, out) = (root.join("src"), root.join("out"));
        write_site(&src, "---\ntitle: X\ndeck: deck/\nimage: deck/thumbnail.png\nimage_alt: Title slide of X\n---\nbody\n");
        write(&src.join("_partials/deck-help.md"), "Press `?`.\n").unwrap();
        write(&out.join("talks/x/deck/index.html"), "").unwrap();
        put_png(&out.join("talks/x/deck/thumbnail.png"), 1200, 630);
        build(&src, &out).unwrap();
        let page = fs::read_to_string(out.join("talks/x/index.html")).unwrap();
        assert!(
            page.contains(
                "<a class=\"deck__banner\" href=\"deck/\"><img src=\"deck/thumbnail.png\" alt=\"Title slide of X. Opens the slides.\">"
            ),
            "{page}"
        );
        assert!(!page.contains("<iframe"), "the deck must not load on the page: {page}");
        assert!(page.contains("Press <code>?</code>."), "{page}");
    }

    #[test]
    fn banner_sits_above_the_title_and_is_the_only_picture() {
        let root = tempdir("deck_above_title");
        let (src, out) = (root.join("src"), root.join("out"));
        write(&src.join("index.md"), "---\ntitle: Site\nurl: https://example.org\n---\n").unwrap();
        write(&src.join("talks/x.md"), "---\ntitle: X\ndeck: deck/\nimage: deck/thumbnail.png\nimage_alt: Title slide of X\n---\nabstract\n").unwrap();
        write(&src.join("_partials/deck-help.md"), "keys\n").unwrap();
        write(&out.join("talks/x/deck/index.html"), "").unwrap();
        put_png(&out.join("talks/x/deck/thumbnail.png"), 1200, 630);
        build(&src, &out).unwrap();
        let page = fs::read_to_string(out.join("talks/x/index.html")).unwrap();
        let (banner, title, help, text) = (
            page.find("class=\"deck__banner\"").unwrap(),
            page.find("<h1>X</h1>").unwrap(),
            page.find("class=\"deck__help\"").unwrap(),
            page.find("<p>abstract</p>").unwrap(),
        );
        assert!(banner < title && title < help && help < text, "{page}");
        assert_eq!(page.matches("<img").count(), 1, "one picture per page: {page}");
        assert!(page.contains("og:image\" content=\"https://example.org/talks/x/deck/thumbnail.png\""), "{page}");
    }

    #[test]
    fn missing_share_image_fails() {
        let root = tempdir("missing_share_image");
        let (src, out) = (root.join("src"), root.join("out"));
        write(&src.join("index.md"), "---\ntitle: Site\n---\n").unwrap();
        write(&src.join("talks/x.md"), "---\ntitle: X\nimage: nope.png\nimage_alt: a picture\n---\n").unwrap();
        let err = build(&src, &out).unwrap_err();
        assert!(err.contains("nope.png"), "{err}");
    }

    /// The bytes of a PNG up to its IHDR chunk, which is all a size reader needs.
    fn put_png(path: &Path, w: u32, h: u32) {
        let mut b = vec![137, 80, 78, 71, 13, 10, 26, 10, 0, 0, 0, 13];
        b.extend_from_slice(b"IHDR");
        b.extend_from_slice(&w.to_be_bytes());
        b.extend_from_slice(&h.to_be_bytes());
        b.extend_from_slice(&[8, 6, 0, 0, 0, 0, 0, 0, 0]);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, b).unwrap();
    }

    /// A site with a url, a twitter handle, a site image, and page `x`.
    fn unfurl_site(root: &Path, page: &str) -> (PathBuf, PathBuf) {
        let (src, out) = (root.join("src"), root.join("out"));
        write(
            &src.join("index.md"),
            "---\ntitle: Site\nurl: https://example.org/\nimage: /images/site.jpg\nimage_alt: A flower\ntwitter: someone\n---\n",
        )
        .unwrap();
        write(&src.join("talks/x.md"), page).unwrap();
        put_png(&out.join("images/site.jpg"), 800, 600);
        (src, out)
    }

    #[test]
    fn page_image_drives_a_large_card_with_size_and_alt() {
        // Fails if a preview loses its size (Facebook and LinkedIn skip
        // unsized images on first share) or X falls back to a small card.
        let root = tempdir("page_image_unfurl");
        let (src, out) = unfurl_site(&root, "---\ntitle: X\nimage: share.png\nimage_alt: Title slide of X\n---\n");
        put_png(&out.join("talks/x/share.png"), 1200, 630);
        build(&src, &out).unwrap();
        let page = fs::read_to_string(out.join("talks/x/index.html")).unwrap();
        for tag in [
            "<link rel=\"canonical\" href=\"https://example.org/talks/x/\">",
            "<meta property=\"og:image\" content=\"https://example.org/talks/x/share.png\">",
            "<meta property=\"og:image:width\" content=\"1200\">",
            "<meta property=\"og:image:height\" content=\"630\">",
            "<meta property=\"og:image:alt\" content=\"Title slide of X\">",
            "<meta name=\"twitter:image:alt\" content=\"Title slide of X\">",
            "<meta name=\"twitter:card\" content=\"summary_large_image\">",
            "<meta name=\"twitter:site\" content=\"@someone\">",
            "<meta name=\"twitter:creator\" content=\"@someone\">",
        ] {
            assert!(page.contains(tag), "missing {tag} in {page}");
        }
    }

    #[test]
    fn page_without_image_shares_the_site_image() {
        let root = tempdir("page_shares_site_image");
        let (src, out) = unfurl_site(&root, "---\ntitle: X\n---\n");
        build(&src, &out).unwrap();
        for file in ["talks/x/index.html", "index.html"] {
            let html = fs::read_to_string(out.join(file)).unwrap();
            assert!(html.contains("og:image\" content=\"https://example.org/images/site.jpg\""), "{html}");
            assert!(html.contains("og:image:width\" content=\"800\""), "{html}");
            assert!(html.contains("og:image:alt\" content=\"A flower\""), "{html}");
        }
    }

    #[test]
    fn page_head_carries_date_and_venue_for_previews() {
        // Fails if Slack loses the Venue and Date rows or the date tag drops.
        let root = tempdir("page_labels");
        let (src, out) = unfurl_site(&root, "---\ntitle: X\ndate: 2026-06-01\nvenue: Some Conf\n---\n");
        build(&src, &out).unwrap();
        let page = fs::read_to_string(out.join("talks/x/index.html")).unwrap();
        for tag in [
            "<meta property=\"article:published_time\" content=\"2026-06-01\">",
            "<meta name=\"twitter:label1\" content=\"Venue\">",
            "<meta name=\"twitter:data1\" content=\"Some Conf\">",
            "<meta name=\"twitter:label2\" content=\"Date\">",
            "<meta name=\"twitter:data2\" content=\"1 June 2026\">",
        ] {
            assert!(page.contains(tag), "missing {tag} in {page}");
        }
    }

    #[test]
    fn header_names_the_site_on_index_and_pages_with_profile_links() {
        let root = tempdir("header_on_every_page");
        let (src, out) = (root.join("src"), root.join("out"));
        write_site(&src, "---\ntitle: X\n---\nbody\n");
        write(
            &src.join("index.md"),
            "---\ntitle: Site\nname: A B\nbio: Does *things*.\nlinks:\n  - label: GitHub\n    url: https://github.com/ab\n  - label: Mastodon\n    url: https://m.example/@ab\n---\n---\ntitle: Card\npage: talks/x.md\n---\nhi\n",
        )
        .unwrap();
        build(&src, &out).unwrap();
        let index = fs::read_to_string(out.join("index.html")).unwrap();
        assert!(index.contains("<h1 class=\"site-header__name\"><a href=\"/\">A B</a></h1>"), "{index}");
        assert!(!index.contains("visually-hidden"), "{index}");
        assert!(index.contains("<p>Does <em>things</em>.</p>"), "{index}");
        let github = index.find("<li><a rel=\"me\" href=\"https://github.com/ab\">GitHub</a></li>").unwrap();
        let mastodon = index.find("<li><a rel=\"me\" href=\"https://m.example/@ab\">Mastodon</a></li>").unwrap();
        assert!(github < mastodon, "links keep their order");
        let page = fs::read_to_string(out.join("talks/x/index.html")).unwrap();
        assert!(page.contains("<p class=\"site-header__name\"><a href=\"/\">A B</a></p>"), "{page}");
        assert_eq!(page.matches("<h1").count(), 1, "a page keeps one h1: {page}");
    }

    #[test]
    fn icon_link_inlines_the_svg_and_keeps_the_label_for_screen_readers() {
        let root = tempdir("header_icon");
        let (src, out) = (root.join("src"), root.join("out"));
        write_site(&src, "---\ntitle: X\n---\nbody\n");
        write(&out.join("icons/g.svg"), "<svg viewBox=\"0 0 24 24\"><path d=\"M0 0\"/></svg>\n").unwrap();
        write(
            &src.join("index.md"),
            "---\ntitle: Site\nname: A\nlinks:\n  - label: GitHub\n    url: https://github.com/a\n    icon: /icons/g.svg\n---\n",
        )
        .unwrap();
        build(&src, &out).unwrap();
        let index = fs::read_to_string(out.join("index.html")).unwrap();
        assert!(
            index.contains("<a rel=\"me\" href=\"https://github.com/a\" aria-label=\"GitHub\" title=\"GitHub\"><span class=\"site-header__icon\" aria-hidden=\"true\"><svg viewBox=\"0 0 24 24\"><path d=\"M0 0\"/></svg></span></a>"),
            "{index}"
        );
    }

    #[test]
    fn icon_that_is_missing_or_not_svg_fails() {
        let root = tempdir("header_icon_bad");
        let (src, out) = (root.join("src"), root.join("out"));
        write_site(&src, "---\ntitle: X\n---\nbody\n");
        let index = "---\ntitle: Site\nname: A\nlinks:\n  - label: G\n    url: https://g.example\n    icon: /g.svg\n---\n";
        write(&src.join("index.md"), index).unwrap();
        assert!(build(&src, &out).unwrap_err().contains("icon for `G`"));
        write(&out.join("g.svg"), "<p>not an icon</p>").unwrap();
        assert!(build(&src, &out).unwrap_err().contains("has no <svg> element"));
    }

    #[test]
    fn no_name_means_no_header_and_header_fields_need_a_name() {
        let root = tempdir("header_needs_name");
        let (src, out) = (root.join("src"), root.join("out"));
        write_site(&src, "---\ntitle: X\n---\nbody\n");
        build(&src, &out).unwrap();
        let index = fs::read_to_string(out.join("index.html")).unwrap();
        assert!(!index.contains("site-header") && index.contains("visually-hidden"), "{index}");
        write(&src.join("index.md"), "---\ntitle: Site\nlinks:\n  - label: G\n    url: https://g.example\n---\n").unwrap();
        assert!(build(&src, &out).unwrap_err().contains("needs `name`"));
    }

    #[test]
    fn site_without_url_gets_a_small_card_and_no_image_tags() {
        // Fails if a relative og:image is written; unfurlers need absolute URLs.
        let root = tempdir("no_url_small_card");
        let (src, out) = (root.join("src"), root.join("out"));
        write(&src.join("index.md"), "---\ntitle: Site\nimage: /images/site.jpg\nimage_alt: A flower\n---\n").unwrap();
        put_png(&out.join("images/site.jpg"), 800, 600);
        build(&src, &out).unwrap();
        let index = fs::read_to_string(out.join("index.html")).unwrap();
        assert!(!index.contains("og:image"), "{index}");
        assert!(index.contains("twitter:card\" content=\"summary\""), "{index}");
    }

    #[test]
    fn image_without_alt_fails_for_site_and_page() {
        let root = tempdir("site_image_needs_alt");
        let (src, out) = (root.join("src"), root.join("out"));
        write(&src.join("index.md"), "---\ntitle: Site\nimage: /images/site.jpg\n---\n").unwrap();
        put_png(&out.join("images/site.jpg"), 800, 600);
        assert!(build(&src, &out).unwrap_err().contains("image_alt"));

        let root = tempdir("page_image_needs_alt");
        let (src, out) = unfurl_site(&root, "---\ntitle: X\nimage: share.png\n---\n");
        put_png(&out.join("talks/x/share.png"), 1200, 630);
        assert!(build(&src, &out).unwrap_err().contains("image_alt"));
    }

    #[test]
    fn unreadable_image_fails() {
        let root = tempdir("unreadable_image");
        let (src, out) = unfurl_site(&root, "---\ntitle: X\nimage: share.png\nimage_alt: a\n---\n");
        write(&out.join("talks/x/share.png"), "not an image").unwrap();
        let err = build(&src, &out).unwrap_err();
        assert!(err.contains("share.png"), "{err}");
        assert!(!out.join("index.html").exists(), "nothing is written when an image cannot be measured");
    }

    fn feed_site(root: &Path, site_extra: &str) -> (PathBuf, PathBuf) {
        let (src, out) = (root.join("src"), root.join("out"));
        write(&src.join("index.md"), &format!("---\ntitle: Site\n{site_extra}---\n")).unwrap();
        write(&src.join("talks/old.md"), "---\ntitle: Old\ndate: 2026-06-01\nfeed: true\n---\n").unwrap();
        write(&src.join("talks/new.md"), "---\ntitle: New & Shiny\ndate: 2026-10-01\nfeed: true\ndescription: d\n---\n").unwrap();
        write(&src.join("talks/hidden.md"), "---\ntitle: Hidden\ndate: 2026-12-01\n---\n").unwrap();
        (src, out)
    }

    #[test]
    fn feed_has_absolute_entry_links_newest_first() {
        let root = tempdir("feed_entries");
        let (src, out) = feed_site(&root, "url: https://example.org/\nauthor: A\n");
        build(&src, &out).unwrap();
        let feed = fs::read_to_string(out.join("feed.xml")).unwrap();
        assert!(feed.contains("<updated>2026-10-01T00:00:00Z</updated>\n<author>"), "{feed}");
        assert!(feed.contains("<id>https://example.org/talks/new/</id>"), "{feed}");
        assert!(feed.contains("<title>New &amp; Shiny</title>"), "{feed}");
        assert!(feed.contains("<summary>d</summary>"), "{feed}");
        assert!(!feed.contains("Hidden"), "{feed}");
        assert!(feed.find("/talks/new/").unwrap() < feed.find("/talks/old/").unwrap());
        let index = fs::read_to_string(out.join("index.html")).unwrap();
        assert!(index.contains("href=\"/feed.xml\""), "{index}");
        assert!(!index.contains("New &amp; Shiny"), "feed pages are not listed on the index: {index}");
    }

    #[test]
    fn feed_without_url_or_author_fails() {
        let root = tempdir("feed_needs_url");
        let (src, out) = feed_site(&root, "author: A\n");
        assert!(build(&src, &out).unwrap_err().contains("url"));
        let root = tempdir("feed_needs_author");
        let (src, out) = feed_site(&root, "url: https://example.org\n");
        assert!(build(&src, &out).unwrap_err().contains("author"));
    }

    #[test]
    fn feed_without_date_fails() {
        let root = tempdir("feed_needs_date");
        let (src, out) = (root.join("src"), root.join("out"));
        write(&src.join("index.md"), "---\ntitle: Site\nurl: https://e.org\nauthor: A\n---\n").unwrap();
        write(&src.join("p.md"), "---\ntitle: P\nfeed: true\n---\n").unwrap();
        assert!(build(&src, &out).unwrap_err().contains("`feed` needs `date`"));
    }

    #[test]
    fn no_feed_pages_means_no_feed() {
        let root = tempdir("no_feed");
        let (src, out) = (root.join("src"), root.join("out"));
        write(&src.join("index.md"), "---\ntitle: Site\n---\n").unwrap();
        build(&src, &out).unwrap();
        assert!(!out.join("feed.xml").exists());
        assert!(!fs::read_to_string(out.join("index.html")).unwrap().contains("feed.xml"));
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
