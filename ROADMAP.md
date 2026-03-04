# Roadmap

## 1. Mermaid dark-mode color fix

Mermaid SVGs are rendered at build time with hardcoded fill/stroke attributes. Tufte CSS switches the page to `#151515` bg / `#ddd` text via `prefers-color-scheme: dark`, but the SVG presentation attributes don't respond — so arrows go invisible and node fills become jarring light boxes.

**Fix:** after each `render_with_options` call in `src/mermaid.rs`, inject a `<style>` block into the SVG string (inserted right after the opening `<svg>` tag). The style contains a single `@media (prefers-color-scheme: dark)` section with attribute-value selectors (`rect[fill="#e8e8e0"]`, `path[stroke="#111111"]`, `text[fill="#111111"]`, etc.) that override the known light-theme colors with their dark equivalents (derived from Tufte's `#151515` bg using the same `lighten_hex` offsets). CSS overrides SVG presentation attributes without `!important`. Only applied when `is_dark(background) == false` (light pages / posts); explicit dark-frontmatter pages already have dark fills and need no injection.

Only `src/mermaid.rs` needs to change.

## 2. Screenshots from the example site

Take screenshots of the built example site (home, post with a mermaid diagram, fiction ToC, slide deck) and add them to the README so visitors can see what Arcadia produces without having to build it themselves.

## 3. Template engine — consider replacing with a library

The current engine (plain substitution + `{{#if}}` conditionals) is intentionally minimal. If further templating features are needed — loops, filters, inheritance, whitespace control — replace it with a proper library rather than extending the hand-rolled engine. Good candidates in the Rust ecosystem: `minijinja` (Jinja2-compatible, small and embeddable) or `tera` (Django-style, more fully featured).

## 4. GitHub release workflow (`.github/workflows/release.yml`)

- Trigger on version tags (`v*`)
- Build release binaries for three targets: `x86_64-unknown-linux-gnu`, `x86_64-apple-darwin`, `aarch64-apple-darwin`
- Use `cargo build --release --target <target>` on the appropriate runner for each
- Upload the resulting `arcadia` (or `arcadia.exe`) binary as a release asset via `softprops/action-gh-release`
- Strip binaries before upload to reduce size
