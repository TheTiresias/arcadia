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
- **Item 6 (chrono)** — Replaced hand-rolled `to_rfc2822` in `src/feeds.rs` with `chrono::NaiveDate`; proper validation, no manual string slicing
- **Item 7 (gray-matter) — skipped** — `gray_matter` (Rust) returns its own `Pod` type and an owned `String` body, not `serde_yaml::Value` and `&str`. Migration would ripple through all callers in `src/content/`. Current `frontmatter.rs` is clean, well-tested, and only 40 lines of logic — not worth the churn.
- **Item 8 (post colors)** — `background_color` / `font_color` frontmatter now supported on posts; `body_style()` and bg/fg forwarded to mermaid renderer. Decks already had this. Template updated (`embed/post.html`).
- **Item 9 (sitemap completeness)** — Sitemap now includes fiction chapter pages, `fiction.html` / `decks.html` / `tags.html` listing indexes, tag pages, and `<lastmod>` on posts with a date. Added `chapter_slugs` to `StoryMeta`.

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
