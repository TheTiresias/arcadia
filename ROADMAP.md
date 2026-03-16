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

## Next — Demo site on GitHub Pages

### 12a. Absorb the HOW_TO docs into the example site ✓

Converted all four `HOW_TO_*.md` files into posts in `example/posts/` (dated 2026-01-01 through 2026-01-04, tagged `docs`). The standalone root files can be removed once the Pages site is live and a redirect URL is known.

### 12b. GitHub Pages deployment

Add a GitHub Actions workflow that runs `arcadia build` on push to `main` and deploys the output to GitHub Pages. Set `base_url` in `arcadia.toml` to the Pages URL so the sitemap and RSS feed have correct absolute URLs. Update the README to link to the live site.

**Files:** `.github/workflows/pages.yml`, `arcadia.toml`, `README.md`
**Depends on:** 12a (so the deployed site has full content)
**Verify:** GitHub Pages site is live and browsable; RSS feed and sitemap contain correct absolute URLs

---

## Suggestions

- **Template engine** — The current engine (plain substitution + `{{#if}}` conditionals) is intentionally minimal. If loops, filters, or inheritance are ever needed, consider `minijinja` (Jinja2-compatible) or `tera` (Django-style). Not worth doing speculatively.
- **Mermaid ET Book font metrics** — The renderer measures glyph widths at build time using system fonts, not ET Book. Node box geometry is therefore slightly imprecise, though invisible in practice (browsers inherit the correct font from the page CSS). Fix would require adding a `register_font_bytes` API to the vendored renderer. Only worth doing if node sizing becomes visibly wrong.
