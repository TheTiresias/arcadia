# How to Write Fiction

Fiction lives in `example/fiction/`. Each story is a directory containing a metadata file and one markdown file per chapter. Create a new story with:

```
arcadia new fiction <slug>
```

This creates `example/fiction/{slug}/story.md` and a starter `chapter-01.md`.

---

## Story Metadata (`story.md`)

Each story directory must contain a `story.md` file. Its frontmatter controls the story-level settings.

```yaml
---
title: Your Story Title
description: A one-line description shown on the fiction index.
background_color: "#1a1a2e"
font_color: "#e0e0e0"
tags: [fantasy, short-story]
---
```

| Field              | Required | Description                                                  |
|--------------------|----------|--------------------------------------------------------------|
| `title`            | Yes      | Displayed on the table of contents and in chapter page titles |
| `description`      | No       | Shown on the main fiction index next to the story link       |
| `background_color` | No       | CSS color value applied to `<body>` on all chapter and ToC pages |
| `font_color`       | No       | CSS color value applied to `<body>` on all chapter and ToC pages |
| `tags`             | No       | List (or comma-separated string) of tags; generates tag pages and shows tag links on the ToC page |

Tags belong to the story, not individual chapters — set them in `story.md` only.

Colors apply consistently across the entire story — all chapters and the table of contents share the same palette.

---

## Chapter Files

Any `.md` file in the story directory other than `story.md` is treated as a chapter.

```yaml
---
title: The First Chapter
order: 1
subtitle: An optional line shown below the chapter title
---
```

| Field      | Required | Description                                              |
|------------|----------|----------------------------------------------------------|
| `title`    | Yes      | Displayed as the chapter `<h1>` and in the ToC listing   |
| `order`    | Yes      | Integer controlling chapter sequence; lower numbers come first |
| `subtitle` | No       | Rendered as a smaller line below the chapter title       |

Chapter filenames don't affect ordering — only the `order` field does.

---

## Navigation

Each chapter page gets a **prev — contents — next** navigation bar at both the top and bottom of the page, built automatically from the chapter order. No configuration needed.

---

## Sections

A horizontal rule `---` in a chapter becomes a `<section>` break, identical to posts. Use it to divide a chapter into named parts.

---

## Sidenotes and Margin Notes

Both are available in chapter prose, using the same syntax as posts.

`^[text]` → numbered sidenote
`>[text]` → unnumbered margin note

```markdown
She had made up her mind.^[Or so she told herself.] The door was already open.
```
