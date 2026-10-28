//! Dead-link warnings for the pages a build wrote.
//!
//! Every `href` and `src` in a written page is checked. A site path must name
//! a file under the output directory (a directory counts when it holds an
//! `index.html`). An `http` or `https` URL must answer below 400. Each URL is
//! fetched once, however many pages use it.

use std::collections::BTreeMap;
use std::path::{Component, Path, PathBuf};
use std::thread;
use std::time::Duration;

/// Where a link points, after its query and fragment are dropped.
#[derive(Debug, PartialEq)]
pub enum Target {
    /// A path under the output directory, without a leading `/`.
    Local(PathBuf),
    Remote(String),
}

/// The `href` and `src` values in `html`, in document order.
pub fn page_links(html: &str) -> Vec<String> {
    let Ok(dom) = tl::parse(html, tl::ParserOptions::default()) else {
        return Vec::new();
    };
    let mut links = Vec::new();
    for node in dom.nodes() {
        let Some(tag) = node.as_tag() else { continue };
        for name in ["href", "src"] {
            if let Some(Some(value)) = tag.attributes().get(name) {
                links.push(value.as_utf8_str().into_owned());
            }
        }
    }
    links
}

/// Classify `link` as written on the page served at `page_url` (a site path
/// ending in `/`). `None` for links with nothing to check: fragments on the
/// same page, `mailto:`, `data:`, and other schemes.
pub fn target(page_url: &str, link: &str) -> Option<Target> {
    let link = link.trim();
    let no_fragment = link.split('#').next().unwrap_or("");
    if no_fragment.starts_with("http://") || no_fragment.starts_with("https://") {
        return Some(Target::Remote(no_fragment.to_string()));
    }
    if let Some(rest) = no_fragment.strip_prefix("//") {
        return Some(Target::Remote(format!("https://{rest}")));
    }
    let path = no_fragment.split('?').next().unwrap_or("");
    if path.is_empty() {
        return None;
    }
    let scheme = path.split_once(':').map(|(s, _)| s);
    if scheme.is_some_and(|s| !s.contains('/')) {
        return None;
    }
    let joined = if let Some(abs) = path.strip_prefix('/') {
        PathBuf::from(abs)
    } else {
        PathBuf::from(page_url.trim_start_matches('/')).join(path)
    };
    let mut clean = PathBuf::new();
    for part in joined.components() {
        match part {
            Component::Normal(p) => clean.push(p),
            Component::ParentDir => {
                clean.pop();
            }
            _ => {}
        }
    }
    Some(Target::Local(clean))
}

fn local_exists(out: &Path, rel: &Path) -> bool {
    let path = out.join(rel);
    path.is_file() || path.join("index.html").is_file()
}

/// The site path a written page is served at: `out/a/b/index.html` is `/a/b/`.
fn served_url(out: &Path, file: &Path) -> Option<String> {
    let rel = file.strip_prefix(out).ok()?;
    if rel.file_name()? != "index.html" {
        return None;
    }
    let dir = rel.parent()?.to_str()?;
    Some(if dir.is_empty() { "/".to_string() } else { format!("/{dir}/") })
}

/// Warnings for dead links in the `written` pages. A URL under `site_url` is
/// checked as a site path, since the build may not be published yet. `fetch`
/// returns the HTTP status of a remote URL, or why it could not be reached.
pub fn check(
    out: &Path,
    written: &[PathBuf],
    site_url: Option<&str>,
    fetch: impl Fn(&str) -> Result<u16, String> + Sync,
) -> Vec<String> {
    let site_url = site_url.map(|u| u.trim_end_matches('/'));
    let mut warnings = Vec::new();
    let mut remote: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for file in written {
        let Some(page) = served_url(out, file) else { continue };
        let Ok(html) = std::fs::read_to_string(file) else { continue };
        for link in page_links(&html) {
            let own = site_url
                .and_then(|base| link.strip_prefix(base))
                .filter(|rest| rest.is_empty() || rest.starts_with(['/', '?', '#']));
            let resolved = match own {
                Some(rest) => target(&page, &format!("/{}", rest.trim_start_matches('/'))),
                None => target(&page, &link),
            };
            match resolved {
                Some(Target::Local(rel)) if !local_exists(out, &rel) => {
                    warnings.push(format!("{page}: dead link {link}: no file {}", out.join(&rel).display()));
                }
                Some(Target::Remote(url)) => {
                    let pages = remote.entry(url).or_default();
                    if !pages.contains(&page) {
                        pages.push(page.clone());
                    }
                }
                _ => {}
            }
        }
    }
    let fetch = &fetch;
    let results: Vec<(String, Result<u16, String>)> = thread::scope(|s| {
        let handles: Vec<_> = remote.keys().map(|url| s.spawn(move || (url.clone(), fetch(url)))).collect();
        handles.into_iter().filter_map(|h| h.join().ok()).collect()
    });
    for (url, result) in results {
        let problem = match result {
            Ok(status) if status < 400 => continue,
            Ok(status) => format!("HTTP {status}"),
            Err(e) => e,
        };
        for page in &remote[&url] {
            warnings.push(format!("{page}: dead link {url}: {problem}"));
        }
    }
    warnings
}

/// HEAD, then GET when HEAD fails: some servers refuse HEAD but serve GET.
/// Redirects are followed.
pub fn http_status(url: &str) -> Result<u16, String> {
    let agent: ureq::Agent = ureq::Agent::config_builder()
        .http_status_as_error(false)
        .timeout_global(Some(Duration::from_secs(15)))
        .user_agent("site-build link check")
        .build()
        .into();
    let head = agent.head(url).call().map(|r| r.status().as_u16());
    match head {
        Ok(status) if status < 400 => Ok(status),
        _ => agent.get(url).call().map(|r| r.status().as_u16()).map_err(|e| e.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn tempdir(name: &str) -> PathBuf {
        let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("target/test-tmp").join(name);
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn collects_href_and_src_from_any_tag() {
        let links = page_links("<a href=\"/a/\">x</a><img src=\"b.png\"><link rel=\"x\" href=\"/c.css\"><a>none</a>");
        assert_eq!(links, vec!["/a/", "b.png", "/c.css"]);
    }

    #[test]
    fn resolves_relative_links_against_the_page_directory() {
        assert_eq!(target("/talks/x/", "deck/"), Some(Target::Local("talks/x/deck".into())));
        assert_eq!(target("/talks/x/", "../y/#top"), Some(Target::Local("talks/y".into())));
        assert_eq!(target("/talks/x/", "/images/a.jpg?v=2"), Some(Target::Local("images/a.jpg".into())));
    }

    #[test]
    fn remote_links_drop_the_fragment_and_protocol_relative_gets_https() {
        assert_eq!(target("/", "https://e.com/a#b"), Some(Target::Remote("https://e.com/a".into())));
        assert_eq!(target("/", "//e.com/s.js"), Some(Target::Remote("https://e.com/s.js".into())));
    }

    #[test]
    fn same_page_fragments_and_other_schemes_are_not_checked() {
        for link in ["#top", "mailto:a@e.com", "data:image/png;base64,AA", "javascript:void(0)", ""] {
            assert_eq!(target("/", link), None, "{link}");
        }
    }

    #[test]
    fn warns_on_missing_local_file_and_failing_remote_once_per_page() {
        let out = tempdir("links_warns");
        fs::create_dir_all(out.join("talks/x")).unwrap();
        fs::write(out.join("ok.png"), "").unwrap();
        fs::write(out.join("talks/x/index.html"), "").unwrap();
        let index = out.join("index.html");
        fs::write(
            &index,
            "<a href=\"/talks/x/\">x</a><img src=\"/ok.png\"><img src=\"/gone.png\">\
             <a href=\"https://e.com/dead\">d</a><a href=\"https://e.com/dead#again\">d</a>\
             <a href=\"https://e.com/live\">l</a><a href=\"https://e.com/down\">n</a>",
        )
        .unwrap();
        let fetch = |url: &str| match url {
            "https://e.com/dead" => Ok(404),
            "https://e.com/down" => Err("connection refused".to_string()),
            _ => Ok(200),
        };
        let mut warnings = check(&out, &[index], None, fetch);
        warnings.sort();
        assert_eq!(warnings.len(), 3, "{warnings:?}");
        assert!(warnings[0].starts_with("/: dead link /gone.png: no file"), "{warnings:?}");
        assert_eq!(warnings[1], "/: dead link https://e.com/dead: HTTP 404");
        assert_eq!(warnings[2], "/: dead link https://e.com/down: connection refused");
    }

    #[test]
    fn files_other_than_index_html_are_not_scanned() {
        let out = tempdir("links_feed");
        let feed = out.join("feed.xml");
        fs::write(&feed, "<link href=\"/missing/\"/>").unwrap();
        assert!(check(&out, &[feed], None, |_| Ok(200)).is_empty());
    }

    #[test]
    fn own_site_urls_are_checked_as_files_without_fetching() {
        let out = tempdir("links_own_site");
        fs::create_dir_all(out.join("talks/x")).unwrap();
        fs::write(out.join("talks/x/index.html"), "").unwrap();
        let index = out.join("index.html");
        fs::write(
            &index,
            "<link rel=\"canonical\" href=\"https://e.github.io/\">\
             <a href=\"https://e.github.io/talks/x/\">x</a><a href=\"https://e.github.io/gone/\">g</a>\
             <a href=\"https://e.github.io.other.com/\">other</a>",
        )
        .unwrap();
        let fetched = std::sync::Mutex::new(Vec::new());
        let warnings = check(&out, &[index], Some("https://e.github.io/"), |url: &str| {
            fetched.lock().unwrap().push(url.to_string());
            Ok(200)
        });
        assert_eq!(warnings.len(), 1, "{warnings:?}");
        assert!(warnings[0].starts_with("/: dead link https://e.github.io/gone/: no file"), "{warnings:?}");
        assert_eq!(*fetched.lock().unwrap(), vec!["https://e.github.io.other.com/"]);
    }

    #[test]
    fn http_status_reports_a_404_from_a_real_server() {
        use std::io::{Read, Write};
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        thread::spawn(move || {
            for stream in listener.incoming().take(2) {
                let mut stream = stream.unwrap();
                let mut buf = [0u8; 1024];
                let _ = stream.read(&mut buf);
                let _ = stream.write_all(b"HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\nConnection: close\r\n\r\n");
            }
        });
        assert_eq!(http_status(&format!("http://{addr}/x")), Ok(404));
    }
}
