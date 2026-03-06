use std::fs;
use std::path::Path;

use anyhow::{Context, Result};

use crate::templates;

/// Scaffold a full site skeleton in `dir`.
pub fn scaffold_site(dir: &Path) -> Result<()> {
    for d in &["example/posts", "example/fiction", "example/decks", "example/resources", "example/images"] {
        fs::create_dir_all(dir.join(d))
            .with_context(|| format!("create {}", d))?;
    }

    fs::write(dir.join("arcadia.toml"), "title = \"My Site\"\n").context("write arcadia.toml")?;

    let sample = "\
---
title: Hello, World
date: 2024-01-01
---

Welcome to your new Arcadia site.^[This is a sidenote. On wide screens it floats into the right margin. Use `^[text]` to create one, or `>[text]` for an unnumbered margin note.]

---

## Diagrams

Arcadia renders [Mermaid](https://mermaid.js.org/) diagrams inline — no JavaScript, no build plugins. Drop a fenced `mermaid` block anywhere in a post:

```mermaid
flowchart LR
    Write --> Build --> Publish
```

Section breaks (`---`) divide a post into `<section>` elements, which is the structural unit Tufte CSS expects.
";
    fs::write(dir.join("example/posts/hello-world.md"), sample)
        .context("write sample post")?;

    println!("Scaffolded new site.");
    Ok(())
}

/// Create a new post stub at `example/posts/<slug>.md`.
pub fn new_post(dir: &Path, slug: &str) -> Result<()> {
    let path = dir.join("example/posts").join(format!("{}.md", slug));
    fs::create_dir_all(path.parent().unwrap()).context("create posts dir")?;
    let content = format!("---\ntitle: {}\ndate: \n---\n\n", slug);
    fs::write(&path, content).with_context(|| format!("write {:?}", path))?;
    println!("Created {:?}", path);
    Ok(())
}

/// Create a new deck stub at `example/decks/<slug>.md`.
pub fn new_deck(dir: &Path, slug: &str) -> Result<()> {
    let path = dir.join("example/decks").join(format!("{}.md", slug));
    fs::create_dir_all(path.parent().unwrap()).context("create decks dir")?;
    let content =
        format!("---\ntitle: {}\n---\n\nSlide one.\n\n---\n\nSlide two.\n", slug);
    fs::write(&path, content).with_context(|| format!("write {:?}", path))?;
    println!("Created {:?}", path);
    Ok(())
}

/// Copy all embedded templates into `{dir}/embed/` so they can be customised locally.
/// Files that already exist are skipped to protect existing edits.
pub fn eject_templates(dir: &Path) -> Result<()> {
    let embed_dir = dir.join("embed");
    fs::create_dir_all(&embed_dir).context("create embed dir")?;

    let templates: &[(&str, &str)] = &[
        ("index.html",        templates::INDEX),
        ("post.html",         templates::POST),
        ("fiction-index.html",templates::FICTION_INDEX),
        ("story-toc.html",    templates::STORY_TOC),
        ("chapter.html",      templates::CHAPTER),
        ("decks-index.html",  templates::DECKS_INDEX),
        ("slide-deck.html",   templates::SLIDE_DECK),
        ("tag-page.html",     templates::TAG_PAGE),
        ("tags-index.html",   templates::TAGS_INDEX),
    ];

    for (name, content) in templates {
        let path = embed_dir.join(name);
        if path.exists() {
            println!("  skipped  embed/{} (already exists)", name);
        } else {
            fs::write(&path, content)
                .with_context(|| format!("write embed/{}", name))?;
            println!("  created  embed/{}", name);
        }
    }

    // Static assets embedded in the binary
    let assets: &[(&str, &str)] = &[
        ("tufte.css", templates::TUFTE_CSS),
    ];
    for (name, content) in assets {
        let path = embed_dir.join(name);
        if path.exists() {
            println!("  skipped  embed/{} (already exists)", name);
        } else {
            fs::write(&path, content)
                .with_context(|| format!("write embed/{}", name))?;
            println!("  created  embed/{}", name);
        }
    }

    Ok(())
}

/// Create a new fiction story skeleton at `example/fiction/<slug>/`.
pub fn new_fiction(dir: &Path, slug: &str) -> Result<()> {
    let story_dir = dir.join("example/fiction").join(slug);
    fs::create_dir_all(&story_dir)
        .with_context(|| format!("create {:?}", story_dir))?;

    let story_md = format!(
        "---\ntitle: {}\ndescription: A new story.\n---\n",
        slug
    );
    fs::write(story_dir.join("story.md"), story_md).context("write story.md")?;

    let chapter_md =
        "---\ntitle: Chapter One\norder: 1\n---\n\nBeginning of the story.\n";
    fs::write(story_dir.join("chapter-01.md"), chapter_md)
        .context("write chapter-01.md")?;

    println!("Created {:?}", story_dir);
    Ok(())
}
