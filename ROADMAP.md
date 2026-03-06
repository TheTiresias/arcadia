# Roadmap

## 1. Screenshots from the example site

Take screenshots of the built example site (home, post with a mermaid diagram, fiction ToC, slide deck) and add them to the README so visitors can see what Arcadia produces without having to build it themselves.

## 2. Template engine — consider replacing with a library

The current engine (plain substitution + `{{#if}}` conditionals) is intentionally minimal. If further templating features are needed — loops, filters, inheritance, whitespace control — replace it with a proper library rather than extending the hand-rolled engine. Good candidates in the Rust ecosystem: `minijinja` (Jinja2-compatible, small and embeddable) or `tera` (Django-style, more fully featured).

## 3. Mermaid diagram rendering quality (`embed/slide-deck.html`, `src/mermaid.rs`)

**3b. Back-edge routing overlap** (`src/mermaid.rs`)

Cyclic diagrams (e.g. `A → B → C → D → A`) produce overlapping arrows because the grid router doesn't carve out enough space to route return edges around existing nodes. The renderer has explicit back-edge support — it routes them around the left or right side of obstacles using `node_spacing` as the avoidance padding (source: `pad = ctx.config.node_spacing.max(30.0)` in `layout.rs`).

The relevant `LayoutConfig` fields and their defaults (all in `src/config.rs` of `mermaid-rs-renderer`):

| Field | Default | Effect |
|---|---|---|
| `node_spacing` | 50.0 | **Primary lever** — directly sets the avoidance pad for back-edge routing; increase to give the router more room to arc around nodes |
| `rank_spacing` | 50.0 | Gap between ranks; increasing helps on dense diagrams |
| `flowchart.routing.occupancy_weight` | 1.2 | How hard the A\* router avoids occupied grid cells; raise to 2–3 to more aggressively route around nodes |
| `flowchart.routing.grid_cell` | 16.0 | Grid resolution for pathfinding; smaller = more routing options at the cost of performance |
| `flowchart.objective.backedge_cross_weight` | 0.65 | Optimizer weight for back-edge crossings during rank assignment |

Recommended starting point: set `node_spacing = 80.0` and `occupancy_weight = 2.5` in the `RenderOptions` constructed in `preprocess()`. These affect all diagrams globally, but extra spacing is harmless on linear graphs.

**3c. Frontmatter overrides for layout parameters** (`src/mermaid.rs`, `src/content/posts.rs`, `src/content/decks.rs`)

Authors writing posts or decks with complex cyclic diagrams should be able to tune layout parameters per-page without forking the binary. Add support for optional frontmatter fields that override the global defaults when constructing `LayoutConfig`:

```yaml
mermaid_node_spacing: 100    # default: 50 — increase for cyclic diagrams with back-edges
mermaid_rank_spacing: 80     # default: 50 — increase for denser graphs
```

These are page-level (not per-diagram), so they apply to all diagrams on that post or deck page. Pass them through to `markdown::render()` alongside the existing `bg`/`fg` color arguments. `occupancy_weight` and `grid_cell` are too internal to expose directly; `node_spacing` and `rank_spacing` are the author-facing levers.

**3d. ET Book font — build-time metrics vs. browser display** (partial limitation)

Two distinct concerns, with different statuses:

*Browser display* — likely already correct. The SVGs are embedded inline in the HTML, and inline SVGs inherit `@font-face` rules from the page's CSS. Since `tufte.css` loads ET Book, diagram text should render with ET Book in browsers despite the font not being available at build time.

*Build-time text measurement* — a remaining limitation. The renderer measures glyph widths at build time to size node boxes, using `fontdb::Database` loaded via `load_system_fonts()`. The `TextMeasurer` and its `fontdb` database are private with no public API for registering additional fonts. As a result, node box geometry is calculated from system serif metrics rather than ET Book metrics — likely close enough to be invisible, but technically imprecise.

The only clean fix is a PR to `mermaid-rs-renderer` adding a `pub fn register_font_bytes(data: &[u8])` function, which would allow arcadia to call it with `include_bytes!("../embed/et-book/...")` before rendering. A fragile workaround would be writing the font files to a temp path that `fontdb` scans at system font load time, but this is OS-dependent and undesirable.

## 4. Deep-link to individual slides (`src/content/decks.rs`, deck template)

- Each slide gets an `id` attribute (`slide-1`, `slide-2`, …) so it can be targeted by a URL fragment (e.g. `decks/my-talk.html#slide-5`)
- The JS navigation logic reads `location.hash` on load and jumps to the matching slide
- When the user navigates between slides, update `location.hash` so the URL stays in sync and the back button works

## 5. Sitemap completeness (`src/sitemap.rs`, `src/content/fiction.rs`)

The sitemap includes posts, story table-of-contents pages, and deck pages, but omits fiction chapter pages, tag pages, and the listing indexes (`fiction.html`, `decks.html`, `tags.html`). To include chapters, `fiction::build` needs to return chapter slugs alongside `StoryMeta` so `sitemap::build` can enumerate them. Add `<lastmod>` tags derived from the `date` frontmatter field where available.

## 6. Custom page colors for posts and decks (`src/content/posts.rs`, `src/content/decks.rs`, `embed/post.html`, `embed/slide-deck.html`)

Fiction pages already support `background_color` and `font_color` frontmatter fields that apply an inline style to `<body>`. The `body_style()` helper in `src/content/mod.rs` that generates this style is already written and shared — posts and decks simply don't call it. Extend both content types to support the same fields:

- In `posts.rs`: extract `background_color` / `font_color` from frontmatter using the existing `body_style()` helper, pass the result to the template renderer as `{{body_style}}`
- In `decks.rs`: same extraction and pass-through
- In `embed/post.html` and `embed/slide-deck.html`: add `{{body_style}}` to the `<body>` opening tag, matching the pattern already used in `embed/chapter.html` and `embed/story-toc.html`

For the Mermaid renderer, `bg` and `fg` should also be forwarded when rendering markdown so diagrams match the page colors, as is already done for fiction chapters.
