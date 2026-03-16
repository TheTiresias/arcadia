use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

use anyhow::{Context, Result};

use crate::content::{tag_slug, DeckMeta, PostMeta, StoryMeta};

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

    let mut push = |path: &str, lastmod: Option<&str>| {
        xml.push_str("  <url><loc>");
        xml.push_str(base);
        xml.push('/');
        xml.push_str(path);
        xml.push_str("</loc>");
        if let Some(date) = lastmod {
            if !date.is_empty() {
                xml.push_str("<lastmod>");
                xml.push_str(date);
                xml.push_str("</lastmod>");
            }
        }
        xml.push_str("</url>\n");
    };

    // Listing indexes
    push("index.html", None);
    push("fiction.html", None);
    push("decks.html", None);
    push("tags.html", None);

    // Posts (with lastmod from date frontmatter)
    for post in posts {
        push(&format!("posts/{}.html", post.slug), Some(&post.date));
    }

    // Fiction: story ToC + each chapter
    for story in stories {
        push(&format!("fiction/{}/index.html", story.slug), None);
        for ch_slug in &story.chapter_slugs {
            push(&format!("fiction/{}/{}.html", story.slug, ch_slug), None);
        }
    }

    // Decks
    for deck in decks {
        push(&format!("decks/{}.html", deck.slug), None);
    }

    // Tag pages — derive from all metas
    let all_tags: BTreeSet<String> = posts
        .iter()
        .flat_map(|p| p.tags.iter().cloned())
        .chain(stories.iter().flat_map(|s| s.tags.iter().cloned()))
        .chain(decks.iter().flat_map(|d| d.tags.iter().cloned()))
        .collect();
    for tag in &all_tags {
        push(&format!("tags/{}.html", tag_slug(tag)), None);
    }

    xml.push_str("</urlset>\n");

    fs::write(out_dir.join("sitemap.xml"), xml).context("write sitemap.xml")?;
    Ok(())
}
