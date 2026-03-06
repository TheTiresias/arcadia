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
