# How to Write Posts

Blog posts live in `src/posts/`. Each file is a single post. Create one with:

```
npm run new:post -- "Your Post Title"
```

---

## Frontmatter

```yaml
---
title: Your Post Title
date: 2026-02-23
subtitle: An optional line shown below the title
tags: [essay, climate]
---
```

| Field      | Required | Description                        |
|------------|----------|------------------------------------|
| `title`    | Yes      | Displayed as the page `<h1>`       |
| `date`     | Yes      | ISO format (`YYYY-MM-DD`); used for chronological sorting on the index |
| `subtitle` | No       | Rendered as a smaller line below the title |
| `tags`     | No       | List (or comma-separated string) of tags; generates tag pages and shows tag links on the post |

---

## Sections

A horizontal rule `---` in your markdown becomes a `<section>` break in the HTML output. Tufte CSS expects content to live inside `<section>` elements, so each rule starts a new visual block. Use them to divide a post into named parts.

```markdown
Opening section prose.

---

## A New Section

Continuing prose here.
```

---

## Sidenotes

Use `^[text]` inline to create a numbered sidenote. On wide screens it floats to the right margin; on narrow screens it collapses inline.

```markdown
Here is a sentence with a sidenote.^[This is the sidenote text.] Prose continues here.
```

The number is generated automatically and increments through the page.

---

## Margin Notes

Use `>[text]` inline to create an unnumbered margin note. Same behaviour as a sidenote but carries no citation number — better for asides that don't warrant attribution.

```markdown
This sentence has a margin note.>[Use these for commentary that doesn't interrupt the sentence.] Prose continues.
```

---

## Other Tufte Elements

**Block quotes** render with Tufte's indented style:

```markdown
> The prose of the world is ordinary. The margins are where the thinking happens.
```

**Code** — both inline `` `code` `` and fenced blocks are supported and render in a monospace face that doesn't shout.

````markdown
```js
const site = 'arcadia'
```
````
