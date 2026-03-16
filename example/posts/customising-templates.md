---
title: Customising Templates
date: 2026-01-04
subtitle: Override any built-in template with a local copy in embed/
tags: [docs]
---

Arcadia's HTML output is driven by ten templates compiled into the binary. You can override any or all of them by placing local copies in `embed/` at the root of your project.

---

## Getting the Default Templates

```
arcadia eject
```

Creates `embed/` and writes all ten templates into it. Files that already exist are skipped, so it is safe to run again after you have already started editing.

---

## How Overrides Work

At the start of every build Arcadia checks `embed/<filename>` in your project directory. If the file exists it is used; otherwise the built-in copy is used. You can override one template and leave the rest untouched.

---

## Template Syntax

### Substitution

Placeholders take the form `{{key}}`. Substitution is plain string replacement. Unknown placeholders are left as-is in the output, which makes partial overrides safe to inspect.

### Conditionals

```html
{{#if key}}...{{/if}}
```

The block is kept (with the tags themselves stripped) when `key` maps to a non-empty value, and removed entirely otherwise. Nesting is not supported.

---

## Common Variables

These variables appear in multiple templates and behave the same everywhere.

| Variable | Value |
|---|---|
| `{{root}}` | Relative path from the output file back to the site root. Always provided by the build system; do not hardcode it. |
| `{{site_title}}` | The `title` field from `arcadia.toml`, or `"Arcadia"` if unset. |
| `{{body_style}}` | Inlined into `<body{{body_style}}>`. Either empty or a leading-space `style="…"` attribute built from `background_color` / `font_color` frontmatter fields. The space is already included — no extra space needed before it in the tag. |

---

## Template Reference

### `index.html` — Home page

Output path: `index.html`

| Variable | Description |
|---|---|
| `{{site_title}}` | Used in `<title>` and the page `<h1>`. |
| `{{root}}` | Always `.`. |
| `{{posts}}` | One `<li>` per post, linking to `posts/{slug}.html`, sorted newest-first. |
| `{{has_fiction}}` | Non-empty when the site has at least one story. Use with `{{#if has_fiction}}`. |
| `{{has_decks}}` | Non-empty when the site has at least one deck. Use with `{{#if has_decks}}`. |

### `post.html` — Individual blog post

Output path: `posts/{slug}.html`

| Variable | Description |
|---|---|
| `{{title}}` | Post title; used in `<title>` and `<h1>`. |
| `{{root}}` | Always `..`. |
| `{{body_style}}` | See Common variables. |
| `{{subtitle}}` | `<p class="subtitle">…</p>`, or empty. |
| `{{date}}` | `<p class="date">…</p>`, or empty. |
| `{{tags}}` | `<p class="tags">…</p>` with tag links, or empty. |
| `{{content}}` | Full markdown body rendered to HTML, wrapped in `<section>` elements. |

### `story-toc.html` — Story table of contents

Output path: `fiction/{slug}/index.html`

| Variable | Description |
|---|---|
| `{{title}}` | Story title. |
| `{{root}}` | Always `../..`. |
| `{{body_style}}` | See Common variables. |
| `{{description}}` | `<p>…</p>`, or empty. |
| `{{tags}}` | Tag links, or empty. |
| `{{chapters}}` | One `<li>` per chapter in reading order. |

### `chapter.html` — Individual chapter

Output path: `fiction/{story-slug}/{chapter-slug}.html`

| Variable | Description |
|---|---|
| `{{title}}` | Chapter title. |
| `{{story_title}}` | Parent story title. |
| `{{root}}` | Always `../..`. |
| `{{body_style}}` | Inherited from `story.md` frontmatter. |
| `{{subtitle}}` | `<p class="subtitle">…</p>`, or empty. |
| `{{nav}}` | `<nav class="chapter-nav">` block with prev/contents/next links. Injected before and after content. |
| `{{content}}` | Chapter body rendered to HTML. |

### `slide-deck.html` — Individual slide deck

Output path: `decks/{slug}.html`

| Variable | Description |
|---|---|
| `{{title}}` | Deck title. |
| `{{root}}` | Always `..`. |
| `{{body_style}}` | See Common variables. |
| `{{tags}}` | Inline tag links, or empty. |
| `{{slides}}` | One `<div class="slide">` per slide. |

**Navigation requirements.** The default template includes JavaScript for prev/next controls. Your markup must preserve: elements with class `slide`, an element with `id="prev"`, an element with `id="next"`, and an element with `id="counter"`.

### `tag-page.html` — Per-tag page

Output path: `tags/{tag-slug}.html`

| Variable | Description |
|---|---|
| `{{tag}}` | The tag name (e.g. `climate`). |
| `{{site_title}}` | Used in `<title>`. |
| `{{root}}` | Always `..`. |
| `{{items}}` | Grouped `<h2>Posts</h2>`, `<h2>Fiction</h2>`, `<h2>Decks</h2>` sections with link lists. |

### `tags-index.html` — Tags listing

Output path: `tags.html`

| Variable | Description |
|---|---|
| `{{site_title}}` | Used in `<title>`. |
| `{{root}}` | Always `.`. |
| `{{tags}}` | One `<li>` per tag with item count, sorted by count descending. |
