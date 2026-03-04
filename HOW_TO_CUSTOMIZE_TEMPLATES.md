# How to Customise Templates

Arcadia's HTML output is driven by nine templates compiled into the binary. You can override any or all of them by placing local copies in `embed/` at the root of your project.

---

## Getting the default templates

```
arcadia eject
```

Creates `embed/` and writes all nine templates into it. Files that already exist are skipped, so it is safe to run again after you have already started editing.

---

## How overrides work

At the start of every build Arcadia checks `embed/<filename>` in your project directory. If the file exists it is used; otherwise the built-in copy is used. You can override one template and leave the rest untouched.

---

## Template syntax

### Substitution

Placeholders take the form `{{key}}`. Substitution is plain string replacement. Unknown placeholders are left as-is in the output, which makes partial overrides safe to inspect.

### Conditionals

```html
{{#if key}}...{{/if}}
```

The block is kept (with the tags themselves stripped) when `key` maps to a non-empty value, and removed entirely otherwise. The body can span multiple lines and contain other `{{key}}` placeholders, which are substituted normally after the conditional pass.

Nesting `{{#if}}` blocks is not supported.

---

## Common variables

These variables appear in multiple templates and behave the same everywhere.

| Variable | Value |
|---|---|
| `{{root}}` | Relative path from the output file back to the site root. Always provided by the build system; do not hardcode it. |
| `{{site_title}}` | The `title` field from `arcadia.toml`, or `"Arcadia"` if unset. |
| `{{body_style}}` | Inlined into `<body{{body_style}}>`. Either empty or a leading-space `style="…"` attribute built from `background_color` / `font_color` frontmatter fields. Because the space is already included, no extra space is needed before it in the tag. |

---

## Template reference

### `index.html` — Home page

Output path: `index.html`

| Variable | Type | Description |
|---|---|---|
| `{{site_title}}` | Plain string | Used in `<title>` and the page `<h1>`. |
| `{{root}}` | Plain string | Always `.`. |
| `{{posts}}` | Pre-rendered HTML | One `<li>` per post, linking to `posts/{slug}.html`, sorted newest-first. |
| `{{has_fiction}}` | Flag | Non-empty when the site has at least one story; empty otherwise. Intended for use with `{{#if has_fiction}}`. |
| `{{has_decks}}` | Flag | Non-empty when the site has at least one deck; empty otherwise. Intended for use with `{{#if has_decks}}`. |

---

### `post.html` — Individual blog post

Output path: `posts/{slug}.html`

| Variable | Type | Description |
|---|---|---|
| `{{title}}` | Plain string | Post title; used in `<title>` and the page `<h1>`. |
| `{{root}}` | Plain string | Always `..`. |
| `{{subtitle}}` | Pre-rendered HTML | `<p class="subtitle">…</p>`, or empty if the frontmatter field is absent. |
| `{{date}}` | Pre-rendered HTML | `<p class="date">…</p>`, or empty if absent. |
| `{{tags}}` | Pre-rendered HTML | `<p class="tags">…</p>` containing `#tag` links, or empty if the post has no tags. |
| `{{content}}` | Pre-rendered HTML | The full markdown body rendered to HTML and wrapped in `<section>` elements. |

---

### `fiction-index.html` — Fiction listing

Output path: `fiction.html`

| Variable | Type | Description |
|---|---|---|
| `{{site_title}}` | Plain string | Used in `<title>`. |
| `{{root}}` | Plain string | Always `.`. |
| `{{stories}}` | Pre-rendered HTML | One `<li>` per story, linking to `fiction/{slug}/index.html`, with a description and chapter count. |

---

### `story-toc.html` — Story table of contents

Output path: `fiction/{slug}/index.html`

| Variable | Type | Description |
|---|---|---|
| `{{title}}` | Plain string | Story title; used in `<title>` and the page `<h1>`. |
| `{{root}}` | Plain string | Always `../..`. |
| `{{body_style}}` | Attribute fragment | See [Common variables](#common-variables). |
| `{{description}}` | Pre-rendered HTML | `<p>…</p>`, or empty if the `description` frontmatter field is absent. |
| `{{tags}}` | Pre-rendered HTML | `<p class="tags">…</p>` containing `#tag` links, or empty if the story has no tags. |
| `{{chapters}}` | Pre-rendered HTML | One `<li>` per chapter in reading order, linking to `{chapter-slug}.html`. Intended for use inside an `<ol>`. |

---

### `chapter.html` — Individual chapter

Output path: `fiction/{story-slug}/{chapter-slug}.html`

| Variable | Type | Description |
|---|---|---|
| `{{title}}` | Plain string | Chapter title; used in `<title>` and the page `<h1>`. |
| `{{story_title}}` | Plain string | Parent story title; used alongside `{{title}}` in `<title>`. |
| `{{root}}` | Plain string | Always `../..`. |
| `{{body_style}}` | Attribute fragment | See [Common variables](#common-variables). Inherited from the story's frontmatter, not the chapter's. |
| `{{subtitle}}` | Pre-rendered HTML | `<p class="subtitle">…</p>`, or empty if absent. |
| `{{nav}}` | Pre-rendered HTML | A `<nav class="chapter-nav">` block with prev/contents/next links. Injected twice — once before and once after `{{content}}`. Both injections receive identical HTML. |
| `{{content}}` | Pre-rendered HTML | The chapter body rendered to HTML and wrapped in `<section>` elements. |

---

### `decks-index.html` — Decks listing

Output path: `decks.html`

| Variable | Type | Description |
|---|---|---|
| `{{site_title}}` | Plain string | Used in `<title>`. |
| `{{root}}` | Plain string | Always `.`. |
| `{{decks}}` | Pre-rendered HTML | One `<li>` per deck, linking to `decks/{slug}.html`. |

---

### `slide-deck.html` — Individual slide deck

Output path: `decks/{slug}.html`

| Variable | Type | Description |
|---|---|---|
| `{{title}}` | Plain string | Deck title; used in `<title>`. |
| `{{root}}` | Plain string | Always `..`. |
| `{{body_style}}` | Attribute fragment | See [Common variables](#common-variables). |
| `{{tags}}` | Pre-rendered HTML | Inline tag links prefixed with `· `, or empty if the deck has no tags. Intended for placement in a header or footer line alongside other inline content. |
| `{{slides}}` | Pre-rendered HTML | One `<div class="slide">` per slide, separated by `---` in the source. |

**Navigation requirements.** The default template includes JavaScript that drives the prev/next controls. If you keep the script, your markup must preserve:

- Elements with class `slide` — the JS selects these with `querySelectorAll('.slide')`.
- An element with `id="prev"` — click triggers previous slide.
- An element with `id="next"` — click triggers next slide.
- An element with `id="counter"` — receives the `"N / total"` text.

You can restyle all of these freely; only the class and IDs need to stay.

---

### `tag-page.html` — Per-tag page

Output path: `tags/{tag-slug}.html`

| Variable | Type | Description |
|---|---|---|
| `{{tag}}` | Plain string | The tag name (e.g. `climate`); used in `<title>` and the page `<h1>`. |
| `{{site_title}}` | Plain string | Used in `<title>`. |
| `{{root}}` | Plain string | Always `..`. |
| `{{items}}` | Pre-rendered HTML | Grouped `<h2>Posts</h2>`, `<h2>Fiction</h2>`, `<h2>Decks</h2>` sections, each followed by a `<ul>` of links. Sections with no entries are omitted entirely. |

---

### `tags-index.html` — Tags listing

Output path: `tags.html`

| Variable | Type | Description |
|---|---|---|
| `{{site_title}}` | Plain string | Used in `<title>`. |
| `{{root}}` | Plain string | Always `.`. |
| `{{tags}}` | Pre-rendered HTML | One `<li>` per tag, linking to `tags/{slug}.html`, sorted by total content count descending. Includes a count in parentheses. |
