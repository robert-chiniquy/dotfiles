# site-build

Builds a one-page card index and any number of article pages from markdown
with YAML frontmatter. The output is plain HTML that links one stylesheet at
`/css/site.css`; the tool writes no CSS or JavaScript of its own. Commit the
output next to the source so the site serves without the tool.

```
site-build <src-dir> <out-dir>
```

## Source layout

```
site/
  index.md                 site frontmatter, then one block per card
  talks/some-talk.md       a page, built to <out>/talks/some-talk/index.html
  _partials/head.html      optional; copied into every <head>
  _partials/deck-help.md   required when any page sets `deck`
```

Every `.md` under the source directory other than `index.md` is a page.
Files and directories whose names start with `_` or `.` are skipped. A page
appears on the index only when a card names it with `page:`. Any other page is
reachable by direct link only.

## Segments

A file is a run of segments. A segment opens with a line that is exactly
`---`, holds YAML up to the next `---` line, and then markdown up to the next
`---` line or the end of the file. Every `---` line is a delimiter, so a
thematic break inside a body is written `***`. Markdown bodies pass raw HTML
through, which is how embeds and canvases go in.

`index.md`: the first segment is the site, with an empty body; each later
segment is one card.

```markdown
---
title: example.github.io
description: some stuff
url: https://example.github.io
image: /images/share.jpg
twitter: example
---
---
title: A talk
page: talks/a-talk.md
---
One line about the talk.
---
title: A photo
image: /images/photo.jpg
image_alt: What the photo shows
link: https://photos.example.com/album
---
Card text in markdown.
```

A page file has exactly one segment:

```markdown
---
title: A talk
date: 2026-10-01
venue: Some Conference 2026
topic: what it is about
description: One sentence for link previews.
deck: deck/
---
Abstract in markdown.
```

## Fields

Site: `title` (required), `description`, `url` (absolute, for `og:url`, the
canonical link, image URLs, and the feed), `image` (site path, the share
image for the index and for any page without its own) with `image_alt`,
`twitter` (handle without `@`), `lang` (default `en`), `author` (feed author;
required once a page sets `feed`), `name` (heading of a header drawn at the
top of every page: the index's `h1`, a paragraph on pages), `bio` (markdown
under the name), `links` (profiles elsewhere, each `label` and `url`, shown in
order as one row with `rel="me"`; an optional `icon`, the site path of an SVG
file, is inlined in place of the label so it takes the text color, and the
label becomes its accessible name and tooltip). `bio` and `links` need `name`.

Card: `title`, `image` with `image_alt` (empty string for decoration),
`page` (source path of a page) or `link` (URL), `background` (CSS color),
`flush` (drop content padding for embeds that fill the card), `header`
(show the card at the right of the index's header instead of in the grid;
one card at most, and the site needs `name`). `page` and `link` need a
`title` to carry the link.

Page: `title` (required), `date` (`YYYY-MM-DD`, shown as `1 October 2026`),
`venue`, `topic`, `description`, `image` (share image, relative to the
page) with `image_alt`, `deck` (directory of an exported slide deck, relative
to the page; needs `image`, which is then shown above the title as a link to
the deck, with the `_partials/deck-help.md` text in an
`<aside class="marginalia deck__help">` after the title; banner, title, and
aside are wrapped in `<div class="tier">`, banner and title in
`<div class="tier__main">`), `feed` (put the page in
`<out>/feed.xml`, an Atom feed, newest first; needs `date`), `youtube` (a
YouTube video URL; its player is embedded from youtube-nocookie.com above the
body, styled by the `.video` class).

## Link previews

Every page gets Open Graph and Twitter Card tags, which Facebook, LinkedIn,
X, Slack, Discord, iMessage, and Mastodon read: title, description, type,
canonical URL, and, when the site has a `url`, the share image with its
width and height (read from the file) and alt text, as a
`summary_large_image` card. A page's `date` is `article:published_time`, and
its `venue` and `date` are the Venue and Date rows Slack shows under the
description. Share images look best at 1200x630.

## Dead links

After writing, every `href` and `src` in the written pages is checked and
each dead one is a warning on stderr; the exit status does not change. A site
path, or a URL under the site `url`, must name a file under the output
directory (a directory counts when it holds an `index.html`). Any other
`http` or `https` URL is fetched once (HEAD, then GET if HEAD fails) and must
answer below HTTP 400.

## Errors

Unknown fields are errors. Nothing is written unless every file parses, every
`page:` names a page, every `deck` directory already has an `index.html`, and
every `image` exists.

## Build and install

```sh
cargo install --path . --root ~/.local
```
