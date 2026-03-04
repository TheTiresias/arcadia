use std::fs;
use std::path::Path;

use anyhow::{Context, Result};

use crate::content::{DeckMeta, PostMeta, StoryMeta};

pub fn build(
    out_dir: &Path,
    posts: &[PostMeta],
    stories: &[StoryMeta],
    decks: &[DeckMeta],
    base_url: &str,
) -> Result<()> {
    let base = base_url.trim_end_matches('/');

    let mut xml = String::from(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
         <urlset xmlns=\"http://www.sitemaps.org/schemas/sitemap/0.9\">\n",
    );

    let mut push = |path: &str| {
        xml.push_str("  <url><loc>");
        xml.push_str(base);
        xml.push('/');
        xml.push_str(path);
        xml.push_str("</loc></url>\n");
    };

    push("index.html");

    for post in posts {
        push(&format!("posts/{}.html", post.slug));
    }
    for story in stories {
        push(&format!("fiction/{}/index.html", story.slug));
    }
    for deck in decks {
        push(&format!("decks/{}.html", deck.slug));
    }

    xml.push_str("</urlset>\n");

    fs::write(out_dir.join("sitemap.xml"), xml).context("write sitemap.xml")?;
    Ok(())
}
