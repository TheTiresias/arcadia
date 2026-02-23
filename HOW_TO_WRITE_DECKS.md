# How to Write Decks

Slide decks live in `src/decks/`. Each file is a single deck. Create one with:

```
npm run new:deck -- "Your Deck Title"
```

---

## Frontmatter

```yaml
---
title: Your Deck Title
background_color: "#1a1a2e"
font_color: "#e0e0e0"
---
```

| Field              | Required | Description                                              |
|--------------------|----------|----------------------------------------------------------|
| `title`            | Yes      | Displayed in the browser tab and on the decks index      |
| `background_color` | No       | CSS color value applied to `<body>` for the entire deck  |
| `font_color`       | No       | CSS color value applied to `<body>` for the entire deck  |

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

Standard markdown works on every slide: headings, lists, bold, italic, inline code, fenced code blocks, and block quotes.

```markdown
## A Code Slide

\```js
function greet(name) {
  return `Hello, ${name}`
}
\```
```

> Note: sidenotes (`^[text]`) and margin notes (`>[text]`) are available syntactically but are not well-suited to the slide format — the narrow slide viewport leaves no room for a margin column. Prefer footnoting ideas directly in the slide text instead.

---

## Colors

`background_color` and `font_color` accept any valid CSS color value:

```yaml
background_color: "#0d1117"   # hex
font_color: "rgb(230, 230, 230)"  # rgb
font_color: "ivory"           # named color
```

The colors are applied to the `<body>` element, so they cover the full viewport including the header and navigation bar.
