# Roadmap

## Blocked

- **Release workflow (PAT)** — The GitHub Actions release workflow fails due to a PAT/permissions issue when uploading release assets via `softprops/action-gh-release`. Build and binary packaging steps succeed; only the upload step is broken. Pinned — do not attempt to fix until PAT situation is resolved.

---

## Done

- **Screenshots** — Added to README (home, post with mermaid diagram, fiction ToC, slide deck)
- **Mermaid 3b** — Back-edge routing overlap fixed (`node_spacing = 80.0`, `occupancy_weight = 2.5`)
- **Mermaid 3c** — Per-page frontmatter overrides for `mermaid_node_spacing` / `mermaid_rank_spacing`
- **Phase 1 cleanup** — `escape_html` → `html-escape`; `copy_dir` consolidation; `f32_field` mermaid helper; `tag_section` deduplication; `write_feed` consolidation
- **Phase 1.5 integration tests** — 31 CLI tests across 6 modules (`build_posts`, `build_fiction`, `build_decks`, `build_tags`, `build_feeds`, `new_command`); run with `cargo test --test integration`

---

## Phase 2 — Library swaps (refactoring, no new features)

Items 6 and 7 touch disjoint files and can run in parallel.

### 6. Replace hand-rolled date formatting with `chrono` (`src/feeds.rs`)

`to_rfc2822()` manually parses `YYYY-MM-DD` strings and constructs an RFC 2822 date by string concatenation. It hardcodes the timezone as `+0000`, does no range validation, and silently produces garbage on any non-conforming input. Replace with `chrono`:

```rust
use chrono::NaiveDate;
NaiveDate::parse_from_str(date, "%Y-%m-%d")
    .map(|d| d.and_hms_opt(0, 0, 0).unwrap().and_utc().to_rfc2822())
    .unwrap_or_default()
```

Add `chrono = { version = "0.4", default-features = false, features = ["std"] }` to `Cargo.toml`.

**Files:** `src/feeds.rs`, `Cargo.toml`
**Depends on:** —
**Can parallelize with:** 7
**Verify:** `cargo test` passes; RSS feed dates in example site are valid RFC 2822 format

### 7. Replace hand-rolled frontmatter parsing with `gray-matter` (`src/frontmatter.rs`)

The current parser manually scans for `\n---` delimiters with bespoke edge-case handling (empty body, closing delimiter at EOF). The `gray-matter` crate handles all of this robustly and is a near drop-in. Verify how much of `frontmatter.rs` would be deleted before committing — if it's close to 100%, the swap is worthwhile.

**Files:** `src/frontmatter.rs`, `Cargo.toml`
**Depends on:** —
**Can parallelize with:** 6
**Verify:** all existing `frontmatter` tests pass or are replaced by equivalent coverage; `cargo test --test integration` produces identical output

---

## Phase 3 — Feature additions

Items 8 and 9 touch disjoint files and can run in parallel.

### 8. Custom page colors for posts and decks (`src/content/posts.rs`, `src/content/decks.rs`, `embed/post.html`, `embed/slide-deck.html`)

Fiction pages already support `background_color` and `font_color` frontmatter fields that apply an inline style to `<body>`. The `body_style()` helper in `src/content/mod.rs` that generates this style is already written and shared — posts and decks simply don't call it. Extend both content types to support the same fields:

- In `posts.rs`: extract `background_color` / `font_color` from frontmatter using the existing `body_style()` helper, pass the result to the template renderer as `{{body_style}}`
- In `decks.rs`: same extraction and pass-through
- In `embed/post.html` and `embed/slide-deck.html`: add `{{body_style}}` to the `<body>` opening tag, matching the pattern already used in `embed/chapter.html` and `embed/story-toc.html`

For the Mermaid renderer, `bg` and `fg` should also be forwarded when rendering markdown so diagrams match the page colors, as is already done for fiction chapters.

**Files:** `src/content/posts.rs`, `src/content/decks.rs`, `embed/post.html`, `embed/slide-deck.html`
**Depends on:** —
**Can parallelize with:** 9
**Verify:** `cargo test` passes; a post with `background_color: "#1a1a1a"` and `font_color: "#eeeeee"` renders with the correct inline body style; mermaid diagrams on that post use matching colors

### 9. Sitemap completeness (`src/sitemap.rs`, `src/content/fiction.rs`)

The sitemap includes posts, story table-of-contents pages, and deck pages, but omits fiction chapter pages, tag pages, and the listing indexes (`fiction.html`, `decks.html`, `tags.html`). To include chapters, `fiction::build` needs to return chapter slugs alongside `StoryMeta` so `sitemap::build` can enumerate them. Add `<lastmod>` tags derived from the `date` frontmatter field where available.

**Files:** `src/sitemap.rs`, `src/content/fiction.rs`, `src/build.rs`
**Depends on:** —
**Can parallelize with:** 8
**Verify:** `cargo test` passes; `example/dist/sitemap.xml` contains URLs for chapter pages, tag pages, and listing index pages; entries with a date field include a `<lastmod>` tag

---

## Phase 4 — Architecture (most disruptive, defer)

These items are high-risk and should not be parallelized with each other. Complete and stabilize one before starting the other.

### 10. Template engine — consider replacing with a library

The current engine (plain substitution + `{{#if}}` conditionals) is intentionally minimal. If further templating features are needed — loops, filters, inheritance, whitespace control — replace it with a proper library rather than extending the hand-rolled engine. Good candidates in the Rust ecosystem: `minijinja` (Jinja2-compatible, small and embeddable) or `tera` (Django-style, more fully featured).

**Files:** `src/templates.rs`, `Cargo.toml`, all template callers in `src/content/`
**Depends on:** all prior phases complete
**Can parallelize with:** 11 (touches disjoint files, but both are high-risk — serialize instead)
**Verify:** `cargo test` passes; full example site build produces byte-for-byte identical HTML output compared to the previous engine

### 11. Mermaid 3d — ET Book font build-time metrics

*Browser display* — likely already correct. The SVGs are embedded inline in the HTML, and inline SVGs inherit `@font-face` rules from the page's CSS. Since `tufte.css` loads ET Book, diagram text should render with ET Book in browsers despite the font not being available at build time.

*Build-time text measurement* — a remaining limitation. The renderer measures glyph widths at build time to size node boxes via `fontdb::Database::load_system_fonts()`. Node box geometry is therefore calculated from system serif metrics rather than ET Book metrics — likely close enough to be invisible in practice, but technically imprecise.

Since `mermaid-rs-renderer` is vendored, the fix can be made directly: add a `pub fn register_font_bytes(data: &[u8])` API to `vendor/mermaid-rs-renderer/src/text_metrics.rs`, then call it with `include_bytes!("../embed/et-book/...")` from `src/mermaid.rs` before rendering.

**Files:** `vendor/mermaid-rs-renderer/src/text_metrics.rs`, `src/mermaid.rs`
**Depends on:** all prior phases complete
**Can parallelize with:** 10 (touches disjoint files, but both are high-risk — serialize instead)
**Verify:** `cargo test` passes; node box widths in rendered SVGs visibly match ET Book glyph metrics

---

## 12. Demo site on GitHub Pages

Consolidate the four `HOW_TO_*.md` files into the example site as proper posts or a dedicated docs section, then publish the built `example/dist/` to GitHub Pages so visitors can browse a live Arcadia site.

**12a. Absorb the HOW_TO docs into the example site**

Convert `HOW_TO_WRITE_POSTS.md`, `HOW_TO_WRITE_FICTION.md`, `HOW_TO_WRITE_DECKS.md`, and `HOW_TO_CUSTOMIZE_TEMPLATES.md` into posts. The example site becomes the canonical reference; the standalone files in the repo root can be removed or replaced with a redirect note pointing to the live site.

**12b. GitHub Pages deployment**

Add a GitHub Actions workflow that runs `arcadia build` on push to `main` and deploys the output to GitHub Pages. Set `base_url` in `arcadia.toml` to the Pages URL so the sitemap and RSS feed have correct absolute URLs. Update the README to link to the live site.

**Files:** `example/`, `.github/workflows/pages.yml`, `README.md`
**Depends on:** —
**Can parallelize with:** 10, 11 (content and infra work, entirely disjoint files)
**Verify:** GitHub Pages site is live and browsable; RSS feed and sitemap contain correct absolute URLs

---

## Suggestions

Opportunistic improvements discovered during development. Not yet scheduled.
