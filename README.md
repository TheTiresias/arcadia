# Arcadia

A Tufte-style static site generator for writing. Supports blog posts, long-form fiction, and slide decks — all from plain markdown.

Built with [Tufte CSS](https://edwardtufte.github.io/tufte-css/), [markdown-it](https://github.com/markdown-it/markdown-it), and [gray-matter](https://github.com/jonschmoll/gray-matter). No framework, no bundler — just a Node.js build script.

---

## Getting Started

```
npm install
npm run build
```

Output goes to `dist/`. Open `dist/index.html` in a browser to view the site.

---

## Using This as a Template

1. **Set your site title** — edit `site.config.js`:
   ```js
   module.exports = {
     title: 'Your Site Name',
     description: 'A short description.',
   }
   ```

2. **Remove the example content** — the repo ships with a sample post, story, and deck so the build works out of the box. Clear them when you're ready to start fresh:
   ```
   npm run clear:examples
   ```

3. **Start writing** — use the `new:post`, `new:story`, and `new:deck` commands to scaffold your content.

---

## Content Types

### Blog Posts

Chronological writing. Lives in `src/posts/`.

```
npm run new:post -- "Your Post Title"
```

See [HOW_TO_WRITE_POSTS.md](HOW_TO_WRITE_POSTS.md) for full details.

### Fiction

Chapter-based long-form writing. Each story is a directory in `src/fiction/` containing a metadata file and one markdown file per chapter. Includes a generated table of contents and prev/next chapter navigation.

```
npm run new:story -- "Your Story Title"
```

See [HOW_TO_WRITE_FICTION.md](HOW_TO_WRITE_FICTION.md) for full details.

### Slide Decks

Presentation slides from markdown. Lives in `src/decks/`. Slides are separated by `---` and navigated with arrow keys or on-screen buttons.

```
npm run new:deck -- "Your Deck Title"
```

See [HOW_TO_WRITE_DECKS.md](HOW_TO_WRITE_DECKS.md) for full details.

---

## Project Structure

```
src/
  posts/          ← blog post markdown files
  fiction/
    {story}/      ← one directory per story
      story.md    ← story metadata
      *.md        ← chapter files
  decks/          ← slide deck markdown files

templates/        ← HTML templates
scripts/          ← scaffolding scripts
build.js          ← build script
dist/             ← generated output (not committed)
```

---

## Tufte Features

Arcadia supports two Tufte-specific markdown extensions in posts and fiction chapters:

**Sidenotes** — numbered, float to the right margin on wide screens:

```markdown
Here is a sentence.^[This is a sidenote.] Prose continues.
```

**Margin notes** — unnumbered, same position:

```markdown
Here is a sentence.>[This is a margin note.] Prose continues.
```

A `---` in posts and fiction chapters becomes a `<section>` break, which is the structural unit Tufte CSS expects.

---

## Tags

Any content type can be tagged by adding a `tags` field to its frontmatter:

```yaml
tags: [essay, climate, fiction]
```

Tags can also be written as a comma-separated string: `tags: essay, climate`.

The build generates:

- `tags/{tag}.html` — all content with that tag, grouped by type
- `tags.html` — master index of every tag with item counts

Tag links appear on post pages, fiction story ToC pages, and in the deck header. The home page links to `tags.html`.

Fiction tags belong to the story, not individual chapters — set them in `story.md`.

---

## Scripts

| Command | Description |
|---|---|
| `npm run build` | Build the full site to `dist/` |
| `npm run serve` | Serve `dist/` locally at `http://localhost:3000` |
| `npm run clear:examples` | Remove the bundled example content |
| `npm run new:post -- "Title"` | Scaffold a new blog post |
| `npm run new:story -- "Title"` | Scaffold a new fiction story with a first chapter |
| `npm run new:deck -- "Title"` | Scaffold a new slide deck |
