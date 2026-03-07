# Roadmap

## 2. Template engine — consider replacing with a library

The current engine (plain substitution + `{{#if}}` conditionals) is intentionally minimal. If further templating features are needed — loops, filters, inheritance, whitespace control — replace it with a proper library rather than extending the hand-rolled engine. Good candidates in the Rust ecosystem: `minijinja` (Jinja2-compatible, small and embeddable) or `tera` (Django-style, more fully featured).

## 3. Mermaid build-time font metrics (`vendor/mermaid-rs-renderer/src/text_metrics.rs`)

The renderer measures glyph widths at build time to size node boxes via `fontdb::Database::load_system_fonts()`. Node box geometry is therefore calculated from system serif metrics rather than ET Book metrics — likely close enough to be invisible in practice, but technically imprecise.

The clean fix is adding a `pub fn register_font_bytes(data: &[u8])` API to the vendored renderer, then calling it with `include_bytes!("../embed/et-book/...")` from `src/mermaid.rs` before rendering. (Browser display is already correct — inline SVGs inherit the page's `@font-face` rules from `tufte.css`.)

## 4. Sitemap completeness (`src/sitemap.rs`, `src/content/fiction.rs`)

The sitemap includes posts, story table-of-contents pages, and deck pages, but omits fiction chapter pages, tag pages, and the listing indexes (`fiction.html`, `decks.html`, `tags.html`). To include chapters, `fiction::build` needs to return chapter slugs alongside `StoryMeta` so `sitemap::build` can enumerate them. Add `<lastmod>` tags derived from the `date` frontmatter field where available.

## 5. Custom page colors for posts and decks (`src/content/posts.rs`, `src/content/decks.rs`, `embed/post.html`, `embed/slide-deck.html`)

Fiction pages already support `background_color` and `font_color` frontmatter fields that apply an inline style to `<body>`. The `body_style()` helper in `src/content/mod.rs` that generates this style is already written and shared — posts and decks simply don't call it. Extend both content types to support the same fields:

- In `posts.rs`: extract `background_color` / `font_color` from frontmatter using the existing `body_style()` helper, pass the result to the template renderer as `{{body_style}}`
- In `decks.rs`: same extraction and pass-through
- In `embed/post.html` and `embed/slide-deck.html`: add `{{body_style}}` to the `<body>` opening tag, matching the pattern already used in `embed/chapter.html` and `embed/story-toc.html`

For the Mermaid renderer, `bg` and `fg` should also be forwarded when rendering markdown so diagrams match the page colors, as is already done for fiction chapters.

## 6. Replace hand-rolled date formatting with `chrono` (`src/feeds.rs`)

`to_rfc2822()` manually parses `YYYY-MM-DD` strings and constructs an RFC 2822 date by string concatenation. It hardcodes the timezone as `+0000`, does no range validation, and silently produces garbage on any non-conforming input. Replace with `chrono`:

```rust
use chrono::NaiveDate;
NaiveDate::parse_from_str(date, "%Y-%m-%d")
    .map(|d| d.and_hms_opt(0, 0, 0).unwrap().and_utc().to_rfc2822())
    .unwrap_or_default()
```

Add `chrono = { version = "0.4", default-features = false, features = ["std"] }` to `Cargo.toml`.

## 7. Replace hand-rolled frontmatter parsing with `gray-matter` (`src/frontmatter.rs`)

The current parser manually scans for `\n---` delimiters with bespoke edge-case handling (empty body, closing delimiter at EOF). The `gray-matter` crate handles all of this robustly and is a near drop-in. Verify how much of `frontmatter.rs` would be deleted before committing — if it's close to 100%, the swap is worthwhile.

## 8. Replace manual `escape_html` with `html-escape` crate (`src/markdown.rs`)

The four-line `escape_html()` function performs substitutions in a specific order (`&` must go first) with no indication that order matters. The `html-escape` crate handles all cases correctly. Small change, eliminates a latent footgun.

## 9. Consolidate `copy_assets` / `copy_dir_recursive` in `build.rs`

Two functions in `src/build.rs` do nearly identical recursive directory copying — `copy_assets` (with mtime-based skip) and `copy_dir_recursive` (unconditional). Merge into one function with an `incremental: bool` parameter and update the three call sites.

## 10. Shared helper for mermaid frontmatter extraction (`src/content/mod.rs`, `posts.rs`, `decks.rs`)

Both `posts.rs` and `decks.rs` contain identical code to extract `mermaid_node_spacing` and `mermaid_rank_spacing` from a frontmatter map:

```rust
meta.get("mermaid_node_spacing").and_then(|v| v.as_f64()).map(|v| v as f32)
```

Move this into a `f32_field(meta, key)` helper in `src/content/mod.rs` alongside the existing `str_field` and `tags_field` helpers.

## 11. Deduplicate tag section rendering in `tags.rs`

`src/content/tags.rs` builds HTML for Posts, Fiction, and Decks tag sections with three copy-pasted `<h2>...<ul>` blocks that differ only in label and URL prefix. Extract a two-argument helper `fn tag_section(label: &str, items: &[(String, String)], url_prefix: &str) -> String` and replace the three blocks.

## Suggestions

Opportunistic improvements discovered during development. Not yet scheduled.

