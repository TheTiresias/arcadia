# Roadmap

## 1. Screenshots from the example site

Take screenshots of the built example site (home, post with a mermaid diagram, fiction ToC, slide deck) and add them to the README so visitors can see what Arcadia produces without having to build it themselves.

## 2. Template engine — consider replacing with a library

The current engine (plain substitution + `{{#if}}` conditionals) is intentionally minimal. If further templating features are needed — loops, filters, inheritance, whitespace control — replace it with a proper library rather than extending the hand-rolled engine. Good candidates in the Rust ecosystem: `minijinja` (Jinja2-compatible, small and embeddable) or `tera` (Django-style, more fully featured).

## 3. GitHub release workflow (`.github/workflows/release.yml`)

- Trigger on version tags (`v*`)
- Build release binaries for three targets: `x86_64-unknown-linux-gnu`, `x86_64-apple-darwin`, `aarch64-apple-darwin`
- Use `cargo build --release --target <target>` on the appropriate runner for each
- Upload the resulting `arcadia` (or `arcadia.exe`) binary as a release asset via `softprops/action-gh-release`
- Strip binaries before upload to reduce size
