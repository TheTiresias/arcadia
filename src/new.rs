use std::fs;
use std::path::Path;

use anyhow::{Context, Result};

/// Scaffold a full site skeleton in `dir`.
pub fn scaffold_site(dir: &Path) -> Result<()> {
    for d in &["example/posts", "example/fiction", "example/decks", "resources", "images"] {
        fs::create_dir_all(dir.join(d))
            .with_context(|| format!("create {}", d))?;
    }

    fs::write(dir.join("arcadia.toml"), "title = \"My Site\"\n").context("write arcadia.toml")?;

    let sample = "---\ntitle: Hello, World\ndate: 2024-01-01\n---\n\nWelcome to your new Arcadia site.\n";
    fs::write(dir.join("example/posts/hello-world.md"), sample)
        .context("write sample post")?;

    println!("Scaffolded new site.");
    println!("Place tufte.css in resources/ before building.");
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
