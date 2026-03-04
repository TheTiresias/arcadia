# Roadmap

## 1. RSS feed (`src/feeds.rs`)

- Generate `dist/feed.xml` (Atom or RSS 2.0) from the posts metadata list
- Include: `title`, `date`, `subtitle` (as description), and a link to the post
- Exclude draft posts
- Use the `rss` crate

## 2. Sitemap (`src/sitemap.rs`)

- Generate `dist/sitemap.xml` from all output URLs (posts, fiction pages, decks)
- Accept a `--base-url <url>` flag on `arcadia build` to produce absolute URLs

## 3. Mermaid diagrams (`src/mermaid.rs`)

- Use [`mermaid-rs-renderer`](https://github.com/1jehuang/mermaid-rs-renderer) — pure Rust, no browser or Node dependency, supports 23 diagram types
- Add as a git dependency with default features disabled (SVG only, ~80 transitive crates vs ~180):
  ```toml
  mermaid-rs-renderer = { git = "https://github.com/1jehuang/mermaid-rs-renderer", tag = "v0.2.0", default-features = false }
  ```
- Pre-processing pass in the markdown pipeline: detect ` ```mermaid ` fenced code blocks
- For each block, call `render(source)` from the crate to produce an SVG string
- Inline the resulting SVG directly into the HTML output (no client-side JS, no subprocess)
- Propagate render errors with the diagram source in the message for easier debugging

## 4. Template engine — consider replacing with a library

The current engine (plain substitution + `{{#if}}` conditionals) is intentionally minimal. If further templating features are needed — loops, filters, inheritance, whitespace control — replace it with a proper library rather than extending the hand-rolled engine. Good candidates in the Rust ecosystem: `minijinja` (Jinja2-compatible, small and embeddable) or `tera` (Django-style, more fully featured).

## 5. GitHub release workflow (`.github/workflows/release.yml`)

- Trigger on version tags (`v*`)
- Build release binaries for three targets: `x86_64-unknown-linux-gnu`, `x86_64-apple-darwin`, `aarch64-apple-darwin`
- Use `cargo build --release --target <target>` on the appropriate runner for each
- Upload the resulting `arcadia` (or `arcadia.exe`) binary as a release asset via `softprops/action-gh-release`
- Strip binaries before upload to reduce size
