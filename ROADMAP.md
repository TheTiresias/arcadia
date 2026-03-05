# Roadmap

## 1. Screenshots from the example site

Take screenshots of the built example site (home, post with a mermaid diagram, fiction ToC, slide deck) and add them to the README so visitors can see what Arcadia produces without having to build it themselves.

## 2. Template engine — consider replacing with a library

The current engine (plain substitution + `{{#if}}` conditionals) is intentionally minimal. If further templating features are needed — loops, filters, inheritance, whitespace control — replace it with a proper library rather than extending the hand-rolled engine. Good candidates in the Rust ecosystem: `minijinja` (Jinja2-compatible, small and embeddable) or `tera` (Django-style, more fully featured).

## 3. Mermaid diagram display in slide decks (`embed/slide-deck.html`)

The SVG output from `mermaid-rs-renderer` has hardcoded dimensions and can overflow the slide horizontally. Fix: add `max-width: 100%; height: auto` to `.slide svg` in the deck template. A secondary limitation is that ET Book fonts are unavailable at build time, so text inside nodes falls back to a system serif — no clean fix without embedding the font as base64 in the SVG.

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
