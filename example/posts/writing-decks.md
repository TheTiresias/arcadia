---
title: How to Write Slide Decks
date: 2026-01-03
subtitle: Slides, navigation, colors, and markdown support
tags: [docs]
---

Slide decks live in `decks/`. Each file is a single deck. Create one with:

```
arcadia new deck <slug>
```

---

## Frontmatter

```yaml
---
title: Your Deck Title
background_color: "#1a1a2e"
font_color: "#e0e0e0"
tags: [talk, research]
---
```

| Field              | Required | Description                                              |
|--------------------|----------|----------------------------------------------------------|
| `title`            | Yes      | Displayed in the browser tab and on the decks index      |
| `background_color` | No       | CSS color value applied to `<body>` for the entire deck  |
| `font_color`       | No       | CSS color value applied to `<body>` for the entire deck  |
| `tags`             | No       | List of tags; generates tag pages and shows tag links in the deck header |

---

## Slides

Each `---` on its own line starts a new slide. The content before the first `---` is the opening slide.

```markdown
---
title: My Deck
---

# My Deck

Opening slide content.

---

## Second Slide

- A bullet point
- Another bullet point

---

## Third Slide

A closing thought.
```

There is no limit on the number of slides.

---

## Navigation

Slides are navigated with:

- **Arrow keys** — left/right
- **Spacebar** — advance to the next slide
- **← / → buttons** — shown at the bottom of the page

A `1 / N` counter tracks position.

---

## Markdown in Slides

Standard markdown works on every slide: headings, lists, bold, italic, inline code, fenced code blocks, and block quotes. Mermaid diagrams are also supported and rendered to inline SVG at build time.

---

## Colors

`background_color` and `font_color` accept any valid CSS color value:

```yaml
background_color: "#0d1117"       # hex
font_color: "rgb(230, 230, 230)"  # rgb
font_color: "ivory"               # named color
```

The colors are applied to the `<body>` element, so they cover the full viewport including the header and navigation bar. Mermaid diagram colors are derived from the page colors automatically.
