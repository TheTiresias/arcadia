# Roadmap

## 1. Screenshots from the example site

Take screenshots of the built example site (home, post with a mermaid diagram, fiction ToC, slide deck) and add them to the README so visitors can see what Arcadia produces without having to build it themselves.

## 2. Template engine — consider replacing with a library

The current engine (plain substitution + `{{#if}}` conditionals) is intentionally minimal. If further templating features are needed — loops, filters, inheritance, whitespace control — replace it with a proper library rather than extending the hand-rolled engine. Good candidates in the Rust ecosystem: `minijinja` (Jinja2-compatible, small and embeddable) or `tera` (Django-style, more fully featured).

## 3. Mermaid diagram rendering quality (`embed/slide-deck.html`, `src/mermaid.rs`)

Three distinct sub-problems:

**3a. SVG overflow in slides** (`embed/slide-deck.html`)

The SVG output has hardcoded pixel dimensions and can overflow the slide boundary horizontally. Fix: add `max-width: 100%; height: auto` to `.slide svg` in the deck template. This is a pure CSS change and does not affect post or fiction chapter pages, where diagrams are already constrained by the Tufte content column.

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

## 4. Eject `tufte.css` (`src/new.rs`)

`arcadia eject` currently writes only the HTML templates to `embed/`. It should also write the embedded `tufte.css` (and any other static resources) so users can customise the stylesheet without forking the binary.

## 5. Deep-link to individual slides (`src/content/decks.rs`, deck template)

- Each slide gets an `id` attribute (`slide-1`, `slide-2`, …) so it can be targeted by a URL fragment (e.g. `decks/my-talk.html#slide-5`)
- The JS navigation logic reads `location.hash` on load and jumps to the matching slide
- When the user navigates between slides, update `location.hash` so the URL stays in sync and the back button works

## 6. GitHub release workflow (`.github/workflows/release.yml`)

- Trigger on version tags (`v*`)
- Build release binaries for three targets: `x86_64-unknown-linux-gnu`, `x86_64-apple-darwin`, `aarch64-apple-darwin`
- Use `cargo build --release --target <target>` on the appropriate runner for each
- Upload the resulting `arcadia` (or `arcadia.exe`) binary as a release asset via `softprops/action-gh-release`
- Strip binaries before upload to reduce size

## 7. Code syntax highlighting (`src/markdown.rs`, `Cargo.toml`)

`pulldown-cmark` emits plain `<code>` blocks with no language decoration. Add syntax highlighting at build time using `syntect` (the most mature Rust option — used by bat, delta, and others). The approach: intercept fenced code blocks during rendering, run them through `syntect`'s HTML generator with an appropriate theme, and emit the highlighted markup. This keeps highlighting fully static with no client-side JavaScript. Theme choice (light/dark, specific palette) should be driven by a field in `arcadia.toml` so users can customise it after ejecting.

## 8. Asset pipeline (`src/build.rs`)

There is no mechanism to include local images or other static files in a site. Add a convention: any files under `{src_dir}/assets/` are copied verbatim into `{out_dir}/assets/` during every build. Authors can then reference images as `/assets/photo.jpg` (or `{{root}}/assets/photo.jpg` in templates). The copy should be recursive, preserve directory structure, and skip files that haven't changed (compare modification times) to keep incremental builds fast.

## 9. Browser hot reload in `arcadia serve` (`src/serve.rs`, templates)

The dev server rebuilds on file changes but does not push a refresh to the browser — authors must reload manually. Fix: add a Server-Sent Events endpoint (e.g. `GET /_reload`) that emits an event after each successful build. Inject a small inline `<script>` into every served page (only when running under `arcadia serve`, not in production builds) that connects to `/_reload` and calls `location.reload()` on receipt. This requires no additional dependencies beyond what `axum` already provides.

## 10. Full content in RSS feed (`src/feeds.rs`)

Feed items currently carry only title, link, date, and subtitle. Most feed readers display nothing useful without body content. Render the full post HTML and include it as the `<content:encoded>` field (or at minimum as `<description>`). Threading the rendered HTML through to `feeds.rs` requires passing it alongside the existing `PostMeta`, or re-rendering at feed-build time.

## 11. Heading anchor links (`src/markdown.rs`)

Headings in rendered output have no `id` attributes, making it impossible to link to a specific section of a post. Post-process the HTML emitted by `pulldown-cmark` to add a slugified `id` to each `<h1>`–`<h6>` and prepend a visually subtle `#` anchor link (hidden until hover, per common convention). The slug should be derived from the heading text using the same `slug` crate already in `Cargo.toml`.

## 12. Sitemap completeness (`src/sitemap.rs`, `src/content/fiction.rs`)

The sitemap includes posts, story table-of-contents pages, and deck pages, but omits fiction chapter pages, tag pages, and the listing indexes (`fiction.html`, `decks.html`, `tags.html`). To include chapters, `fiction::build` needs to return chapter slugs alongside `StoryMeta` so `sitemap::build` can enumerate them. Add `<lastmod>` tags derived from the `date` frontmatter field where available.

## 13. Watch `embed/` in `arcadia serve` (`src/serve.rs`)

`serve.rs` only watches `src_dir` for changes. Modifications to ejected templates in `embed/` require a full server restart to take effect. Add a second watcher on the `embed/` directory (if it exists) that triggers the same rebuild logic.

## 14. RSS feeds for fiction and decks (`src/feeds.rs`, `src/content/`)

The RSS feed covers posts only. Fiction and decks could have their own feeds (`fiction-feed.xml`, `decks-feed.xml`) generated alongside `feed.xml`. For fiction, the natural unit is a new story or chapter; for decks, a new deck. The `rss` crate is already a dependency so this is purely additive.

## 15. Generate a `404.html` (`src/build.rs`, `embed/`)

Many static hosts (GitHub Pages, Netlify, Vercel) will serve `404.html` from the output root as the not-found page if it exists. Add a `404.html` template to `embed/` and generate it during every build. Content can be minimal — site title, a short message, and a link home — but it avoids authors hitting the host's default unstyled error page.

## 16. Footnotes (`src/markdown.rs`)

`pulldown-cmark` supports standard Markdown footnotes (`[^1]` / `[^1]: text`) via `Options::ENABLE_FOOTNOTES`, but the option is not set. Footnotes are semantically distinct from sidenotes: they are referenced inline and collected at the bottom of the document, while sidenotes float in the margin. Enabling the option costs nothing; rendering them in a Tufte-appropriate style (small text, bottom of article) may require a small CSS addition to `tufte.css`.

## 17. Open Graph meta tags (`embed/*.html`)

None of the HTML templates include `<meta property="og:*">` or `<meta name="twitter:*">` tags. Without them, links shared on social platforms show no title, description, or image preview. At minimum, add `og:title`, `og:description`, and `og:url` to the post, chapter, and deck templates, populated from existing frontmatter fields (`title`, `subtitle`/`description`, and the page's canonical URL). Canonical URL generation requires `base_url` to be set in `arcadia.toml`.

## 18. Custom page colors for posts and decks (`src/content/posts.rs`, `src/content/decks.rs`, `embed/post.html`, `embed/slide-deck.html`)

Fiction pages already support `background_color` and `font_color` frontmatter fields that apply an inline style to `<body>`. The `body_style()` helper in `src/content/mod.rs` that generates this style is already written and shared — posts and decks simply don't call it. Extend both content types to support the same fields:

- In `posts.rs`: extract `background_color` / `font_color` from frontmatter using the existing `body_style()` helper, pass the result to the template renderer as `{{body_style}}`
- In `decks.rs`: same extraction and pass-through
- In `embed/post.html` and `embed/slide-deck.html`: add `{{body_style}}` to the `<body>` opening tag, matching the pattern already used in `embed/chapter.html` and `embed/story-toc.html`

For the Mermaid renderer, `bg` and `fg` should also be forwarded when rendering markdown so diagrams match the page colors, as is already done for fiction chapters.
