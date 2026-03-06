# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What Arcadia Is

A Tufte-style static site generator for writing. One binary, no JavaScript in output. Supports blog posts, long-form fiction, and slide decks from plain markdown. Mermaid diagrams are pre-rendered to inline SVG at build time.

## Commands

```bash
cargo build                  # debug build
cargo build --release        # release build
cargo test                   # run all tests
cargo test -p arcadia <name> # run a single test by name
cargo clippy                 # lint
```

To test against the example site:
```bash
cargo run -- build --project example   # build to example/dist/
cargo run -- serve --project example   # dev server with live reload on :3000
```

## Architecture

The binary exposes five CLI commands (`new`, `build`, `serve`, `eject`, `clean`). The main pipeline lives in `src/build.rs`:

1. Three content pipelines run in parallel (posts, fiction, decks)
2. Tags, index pages, 404, RSS feeds, and sitemap are generated after
3. Static assets (`resources/`, `images/`, `assets/`) are copied to `dist/`
4. `tufte.css` is always written from the binary (or ejected override) — never from the user's `resources/`

### Markdown rendering (`src/markdown.rs`)

`render(input, bg, fg, node_spacing, rank_spacing)` runs five stages:

1. `mermaid::preprocess` — replaces ` ```mermaid ``` ` blocks with inline SVG
2. `preprocess` — expands `^[sidenote]` and `>[margin note]` to Tufte HTML
3. pulldown-cmark — renders markdown to HTML
4. `syntax_highlight` — intercepts fenced code blocks, replaces with syntect output
5. `add_heading_anchors` — adds `id` and `#` anchor links to every heading

### Templates (`src/templates.rs` + `embed/`)

All HTML templates are embedded in the binary via `include_str!()`. `Templates::load()` checks for a local `embed/` override at the project root first. The template engine supports `{{key}}` substitution and `{{#if key}}...{{/if}}` conditionals only — no loops. Run `arcadia eject` to copy templates to `embed/` for customization.

### Content types (`src/content/`)

- `posts.rs` — scans `posts/*.md`; supports `draft`, `date`, `subtitle`, `background_color`, `font_color`, `mermaid_node_spacing`, `mermaid_rank_spacing` frontmatter
- `fiction.rs` — scans `fiction/{story}/`; `story.md` holds story-level metadata; chapters are numbered via `order` frontmatter; each story gets a ToC + per-chapter pages with prev/next nav
- `decks.rs` — scans `decks/*.md`; slides split on `\n---\n`; supports same color + mermaid frontmatter as posts
- `tags.rs` — aggregates tags across all content types; generates `tags.html` and `tags/{tag}.html`

### Mermaid (`src/mermaid.rs`)

Uses `mermaid-rs-renderer` (git dep, tag `v0.2.0`). Default layout: `node_spacing = 80.0`, `occupancy_weight = 2.5`. Per-page overrides via frontmatter `mermaid_node_spacing` / `mermaid_rank_spacing`.

## Collaboration & Workflow

- **One unit at a time.** Implement the smallest coherent chunk, then stop and check in. Never chain multiple steps without a pause.
- **Propose before building.** For anything beyond a trivial edit, describe the approach first and get explicit sign-off before writing code.
- **Recommend, don't just list options.** When there are multiple valid approaches, present the tradeoffs and make a clear recommendation with justification.
- **Flag surprises immediately.** If something unexpected turns up mid-implementation, stop and report rather than resolving it silently.
- **Don't add uninvited.** No related improvements, refactors, or "while I'm here" extras unless asked. If you discover something worth doing, add it to the **Suggestions** section of ROADMAP.md and move on.
- **Ask, don't assume the next step.** After finishing a step, check in rather than automatically proceeding.
- **Prefer the narrowest diff.** If the task can be done touching one file, don't touch three.
