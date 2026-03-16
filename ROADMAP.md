# Roadmap

## Done

- **Screenshots** — Added to README (home, post with mermaid diagram, fiction ToC, slide deck)
- **Mermaid 3b** — Back-edge routing overlap fixed (`node_spacing = 80.0`, `occupancy_weight = 2.5`)
- **Mermaid 3c** — Per-page frontmatter overrides for `mermaid_node_spacing` / `mermaid_rank_spacing`

---

## Phase 1 — Cleanup (no new features) ✓ Done

All five items in this phase touch disjoint files and can be assigned to agents in parallel.

### 1. Replace manual `escape_html` with `html-escape` crate (`src/markdown.rs`)

The four-line `escape_html()` function performs substitutions in a specific order (`&` must go first) with no indication that order matters. The `html-escape` crate handles all cases correctly. Small change, eliminates a latent footgun.

**Files:** `src/markdown.rs`, `Cargo.toml`
**Depends on:** —
**Can parallelize with:** 2, 3, 4, 5
**Verify:** `cargo test` passes; `cargo build --release` succeeds

### 2. Consolidate `copy_assets` / `copy_dir_recursive` in `build.rs`

Two functions in `src/build.rs` do nearly identical recursive directory copying — `copy_assets` (with mtime-based skip) and `copy_dir_recursive` (unconditional). Merge into one function with an `incremental: bool` parameter and update the three call sites.

**Files:** `src/build.rs`
**Depends on:** —
**Can parallelize with:** 1, 3, 4, 5
**Verify:** `cargo test` passes; `cargo run -- build --project example` completes and produces output in `example/dist/`

### 3. Shared helper for mermaid frontmatter extraction (`src/content/mod.rs`, `posts.rs`, `decks.rs`)

Both `posts.rs` and `decks.rs` contain identical code to extract `mermaid_node_spacing` and `mermaid_rank_spacing` from a frontmatter map:

```rust
meta.get("mermaid_node_spacing").and_then(|v| v.as_f64()).map(|v| v as f32)
```

Move this into a `f32_field(meta, key)` helper in `src/content/mod.rs` alongside the existing `str_field` and `tags_field` helpers.

**Files:** `src/content/mod.rs`, `src/content/posts.rs`, `src/content/decks.rs`
**Depends on:** —
**Can parallelize with:** 1, 2, 4, 5
**Verify:** `cargo test` passes; mermaid frontmatter overrides still apply correctly in example site

### 4. Deduplicate tag section rendering in `tags.rs`

`src/content/tags.rs` builds HTML for Posts, Fiction, and Decks tag sections with three copy-pasted `<h2>...<ul>` blocks that differ only in label and URL prefix. Extract a helper `fn tag_section(label: &str, items: &[(String, String)], url_prefix: &str) -> String` and replace the three blocks.

**Files:** `src/content/tags.rs`
**Depends on:** —
**Can parallelize with:** 1, 2, 3, 5
**Verify:** `cargo test` passes; tag index and per-tag pages render correctly in example site

### 5. Consolidate feed generation functions in `feeds.rs`

`build()`, `build_fiction()`, and `build_decks()` follow the same pattern: iterate items → build `ItemBuilder` → create `ChannelBuilder` → write file. Extract a shared generic function parameterized by a closure or trait, eliminating ~70 lines of copy-paste. The unified function's signature should accept a title, description, base URL, output path, and an iterator of `(title, link, date, description)` tuples.

**Files:** `src/feeds.rs`
**Depends on:** —
**Can parallelize with:** 1, 2, 3, 4
**Verify:** `cargo test` passes; all three RSS feeds (`feed.xml`, `fiction-feed.xml`, `decks-feed.xml`) are generated in example site with correct items

---

## Phase 1.5 — Integration test suite (before continuing with Phase 2)

The current test suite is entirely unit tests against internal functions. There is no coverage of the actual CLI behavior: whether `arcadia build` produces correct HTML, whether drafts are excluded, whether prev/next navigation is wired up in fiction chapters, etc. The library swaps in Phase 2 are exactly the kind of change that can silently break output — integration tests should be in place first.

### 0. Add integration test scaffolding

Add `assert_cmd`, `predicates`, and `tempfile` as dev-dependencies. Create the `tests/` directory with fixture projects and a test module per command.

**New files/dirs:**
```
tests/
  integration/
    mod.rs          — shared helpers (build_fixture, fixture_path, etc.)
    build_posts.rs
    build_fiction.rs
    build_decks.rs
    build_tags.rs
    build_feeds.rs
    new_command.rs
  fixtures/
    simple_post/
      arcadia.toml
      src/posts/hello.md        — title, date, subtitle, tags, body with heading
    draft_post/
      arcadia.toml
      src/posts/published.md
      src/posts/draft.md        — draft: true
    fiction_story/
      arcadia.toml
      src/fiction/my-story/
        story.md                — title, description, tags
        ch1.md                  — order: 1
        ch2.md                  — order: 2
        ch3.md                  — order: 3
    simple_deck/
      arcadia.toml
      src/decks/my-deck.md      — three slides split on \n---\n
    tagged_content/
      arcadia.toml
      src/posts/rust-post.md    — tags: [rust, programming]
      src/fiction/tagged-story/
        story.md                — tags: [rust]
        ch1.md
    with_base_url/
      arcadia.toml              — base_url = "https://example.com"
      src/posts/hello.md
```

**`Cargo.toml` additions:**
```toml
[dev-dependencies]
assert_cmd = "2"
predicates = "3"
tempfile = "3"
```

**Shared helper sketch (`tests/integration/mod.rs`):**
```rust
use std::path::{Path, PathBuf};
use assert_cmd::Command;
use tempfile::TempDir;

pub fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name)
}

pub fn build_fixture(fixture: &str) -> (TempDir, PathBuf) {
    let tmp = tempfile::tempdir().unwrap();
    // copy fixture into tmp so tests don't write into the repo
    copy_dir_all(&fixture_path(fixture), tmp.path()).unwrap();
    let dist = tmp.path().join("dist");
    Command::cargo_bin("arcadia").unwrap()
        .args(["build", "--project", tmp.path().to_str().unwrap()])
        .assert()
        .success();
    (tmp, dist)
}
```

**Files:** `tests/`, `Cargo.toml`
**Depends on:** —
**Can parallelize with:** nothing — must be done before Phase 2
**Verify:** `cargo test` includes integration tests and they pass; `cargo test --test integration` runs only the CLI tests

---

### Test cases to implement

Each test should be independent (own fixture, own `TempDir`).

#### `build_posts.rs`

| Test | Fixture | Assert |
|------|---------|--------|
| `post_renders_html` | `simple_post` | `dist/posts/hello.html` exists |
| `post_contains_title` | `simple_post` | file contains `<h1>` with post title |
| `post_contains_date` | `simple_post` | file contains date string |
| `post_contains_subtitle` | `simple_post` | file contains subtitle text |
| `draft_excluded_by_default` | `draft_post` | `dist/posts/draft.html` does not exist |
| `draft_included_with_flag` | `draft_post` | run with `--drafts`; `dist/posts/draft.html` exists |
| `post_index_lists_post` | `simple_post` | `dist/index.html` contains link to post |

#### `build_fiction.rs`

| Test | Fixture | Assert |
|------|---------|--------|
| `story_toc_rendered` | `fiction_story` | `dist/fiction/my-story/index.html` exists |
| `chapter_pages_rendered` | `fiction_story` | `dist/fiction/my-story/ch1.html` … `ch3.html` exist |
| `chapter_has_next_link` | `fiction_story` | `ch1.html` contains link to `ch2.html` |
| `chapter_has_prev_link` | `fiction_story` | `ch2.html` contains link to `ch1.html` |
| `first_chapter_no_prev` | `fiction_story` | `ch1.html` does not contain prev link |
| `last_chapter_no_next` | `fiction_story` | `ch3.html` does not contain next link |
| `fiction_index_lists_story` | `fiction_story` | `dist/fiction.html` contains story title |

#### `build_decks.rs`

| Test | Fixture | Assert |
|------|---------|--------|
| `deck_renders_html` | `simple_deck` | `dist/decks/my-deck.html` exists |
| `deck_has_three_slides` | `simple_deck` | file contains three `<div class="slide"` blocks |
| `decks_index_lists_deck` | `simple_deck` | `dist/decks.html` contains link to deck |

#### `build_tags.rs`

| Test | Fixture | Assert |
|------|---------|--------|
| `tag_index_rendered` | `tagged_content` | `dist/tags.html` exists and contains `#rust` |
| `tag_page_rendered` | `tagged_content` | `dist/tags/rust.html` exists |
| `tag_page_lists_post` | `tagged_content` | `dist/tags/rust.html` contains link to post |
| `tag_page_lists_story` | `tagged_content` | `dist/tags/rust.html` contains link to story |

#### `build_feeds.rs`

| Test | Fixture | Assert |
|------|---------|--------|
| `feeds_not_generated_without_base_url` | `simple_post` | `dist/feed.xml` does not exist |
| `post_feed_generated` | `with_base_url` | `dist/feed.xml` exists, is valid XML |
| `post_feed_contains_item` | `with_base_url` | `feed.xml` contains post title and link |
| `fiction_feed_generated` | `with_base_url` (add story) | `dist/fiction-feed.xml` exists |
| `decks_feed_generated` | `with_base_url` (add deck) | `dist/decks-feed.xml` exists |

#### `new_command.rs`

| Test | Assert |
|------|--------|
| `new_scaffolds_dirs` | `posts/`, `fiction/`, `decks/`, `images/`, `resources/`, `assets/` exist |
| `new_creates_config` | `arcadia.toml` exists and contains `title` key |
| `new_post_creates_file` | `arcadia new post "My Post"` creates a `.md` in `posts/` |
| `new_deck_creates_file` | `arcadia new deck "My Deck"` creates a `.md` in `decks/` |

---

## Phase 2 — Library swaps (refactoring, no new features)

Items 6 and 7 touch disjoint files and can run in parallel, but both depend on Phase 1 being complete.

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
**Depends on:** 5 (consolidate feeds first so chrono is added once to the unified function)
**Can parallelize with:** 7
**Verify:** `cargo test` passes; RSS feed dates in example site are valid RFC 2822 format

### 7. Replace hand-rolled frontmatter parsing with `gray-matter` (`src/frontmatter.rs`)

The current parser manually scans for `\n---` delimiters with bespoke edge-case handling (empty body, closing delimiter at EOF). The `gray-matter` crate handles all of this robustly and is a near drop-in. Verify how much of `frontmatter.rs` would be deleted before committing — if it's close to 100%, the swap is worthwhile.

**Files:** `src/frontmatter.rs`, `Cargo.toml`
**Depends on:** —
**Can parallelize with:** 6
**Verify:** all existing `frontmatter` tests pass or are replaced by equivalent coverage; `cargo run -- build --project example` produces identical output

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
**Depends on:** 3 (mermaid helper cleans up `posts.rs`/`decks.rs` first, making this a narrower diff)
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

### 11. Mermaid 3d — ET Book font build-time metrics (requires upstream PR)

*Browser display* — likely already correct. The SVGs are embedded inline in the HTML, and inline SVGs inherit `@font-face` rules from the page's CSS. Since `tufte.css` loads ET Book, diagram text should render with ET Book in browsers despite the font not being available at build time.

*Build-time text measurement* — a remaining limitation. The renderer measures glyph widths at build time to size node boxes, using `fontdb::Database` loaded via `load_system_fonts()`. The `TextMeasurer` and its `fontdb` database are private with no public API for registering additional fonts. As a result, node box geometry is calculated from system serif metrics rather than ET Book metrics — likely close enough to be invisible, but technically imprecise.

The only clean fix is a PR to `mermaid-rs-renderer` adding a `pub fn register_font_bytes(data: &[u8])` function, which would allow arcadia to call it with `include_bytes!("../embed/et-book/...")` before rendering.

**Files:** `src/mermaid.rs` (after upstream PR is merged and dep is updated)
**Depends on:** upstream PR to `mermaid-rs-renderer` merged
**Can parallelize with:** 10 (touches disjoint files, but both are high-risk — serialize instead)
**Verify:** `cargo test` passes; node box widths in rendered SVGs match ET Book glyph metrics

---

## Suggestions

Opportunistic improvements discovered during development. Not yet scheduled.
