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
gets a card on the index only when a card names it with `page:`, so a page
with no card is reachable by direct link only.

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

Site: `title` (required), `description`, `url` (absolute, for `og:url`),
`image` (site path, for `og:image`), `twitter` (handle without `@`), `lang`
(default `en`).

Card: `title`, `image` with `image_alt` (empty string for decoration),
`page` (source path of a page) or `link` (URL), `background` (CSS color),
`flush` (drop content padding for embeds that fill the card). `page` and
`link` need a `title` to carry the link.

Page: `title` (required), `date` (`YYYY-MM-DD`, shown as `1 October 2026`),
`venue`, `topic`, `description`, `deck` (directory of an exported slide deck,
relative to the page; framed in an iframe above the body, followed by the
`_partials/deck-help.md` text), `banner` (a PNG relative to the page, shown
above the title, linked to the deck when there is one, and used as the
page's `og:image`).

Unknown fields are errors. Nothing is written unless every file parses, every
`page:` names a page, every `deck` directory already has an `index.html`, and
every `banner` is a PNG that exists.

## Build and install

```sh
cargo install --path . --root ~/.local
```
