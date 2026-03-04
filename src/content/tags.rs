use std::collections::HashMap;
use std::fs;
use std::path::Path;

use anyhow::{Context, Result};

use super::{tag_slug, DeckMeta, PostMeta, StoryMeta};
use crate::templates::{self, Templates};

struct TagEntry {
    name: String,
    posts: Vec<(String, String)>,   // (slug, title)
    stories: Vec<(String, String)>, // (slug, title)
    decks: Vec<(String, String)>,   // (slug, title)
}

pub fn build(
    out_dir: &Path,
    posts: &[PostMeta],
    stories: &[StoryMeta],
    decks: &[DeckMeta],
    site_title: &str,
    tmpl: &Templates,
) -> Result<()> {
    let tags_out = out_dir.join("tags");
    fs::create_dir_all(&tags_out).context("create dist/tags")?;

    let mut map: HashMap<String, TagEntry> = HashMap::new();

    for post in posts {
        for tag in &post.tags {
            let slug = tag_slug(tag);
            let entry = map.entry(slug).or_insert_with(|| TagEntry {
                name: tag.clone(),
                posts: Vec::new(),
                stories: Vec::new(),
                decks: Vec::new(),
            });
            entry.posts.push((post.slug.clone(), post.title.clone()));
        }
    }

    for story in stories {
        for tag in &story.tags {
            let slug = tag_slug(tag);
            let entry = map.entry(slug).or_insert_with(|| TagEntry {
                name: tag.clone(),
                posts: Vec::new(),
                stories: Vec::new(),
                decks: Vec::new(),
            });
            entry.stories.push((story.slug.clone(), story.title.clone()));
        }
    }

    for deck in decks {
        for tag in &deck.tags {
            let slug = tag_slug(tag);
            let entry = map.entry(slug).or_insert_with(|| TagEntry {
                name: tag.clone(),
                posts: Vec::new(),
                stories: Vec::new(),
                decks: Vec::new(),
            });
            entry.decks.push((deck.slug.clone(), deck.title.clone()));
        }
    }

    // Generate per-tag pages
    for (slug, entry) in &map {
        let mut items = String::new();

        if !entry.posts.is_empty() {
            items.push_str("<h2>Posts</h2>\n<ul>\n");
            for (ps, pt) in &entry.posts {
                items.push_str(&format!(
                    "  <li><a href=\"../posts/{}.html\">{}</a></li>\n",
                    ps, pt
                ));
            }
            items.push_str("</ul>\n");
        }

        if !entry.stories.is_empty() {
            items.push_str("<h2>Fiction</h2>\n<ul>\n");
            for (ss, st) in &entry.stories {
                items.push_str(&format!(
                    "  <li><a href=\"../fiction/{}/index.html\">{}</a></li>\n",
                    ss, st
                ));
            }
            items.push_str("</ul>\n");
        }

        if !entry.decks.is_empty() {
            items.push_str("<h2>Decks</h2>\n<ul>\n");
            for (ds, dt) in &entry.decks {
                items.push_str(&format!(
                    "  <li><a href=\"../decks/{}.html\">{}</a></li>\n",
                    ds, dt
                ));
            }
            items.push_str("</ul>\n");
        }

        let html = templates::render(
            &tmpl.tag_page,
            &[
                ("tag", &entry.name),
                ("site_title", site_title),
                ("root", ".."),
                ("items", &items),
            ],
        );

        let out_path = tags_out.join(format!("{}.html", slug));
        fs::write(&out_path, html)
            .with_context(|| format!("write {:?}", out_path))?;
    }

    // Generate tags index sorted by total count desc
    let mut sorted: Vec<(&String, &TagEntry)> = map.iter().collect();
    sorted.sort_by(|a, b| {
        let ca = a.1.posts.len() + a.1.stories.len() + a.1.decks.len();
        let cb = b.1.posts.len() + b.1.stories.len() + b.1.decks.len();
        cb.cmp(&ca).then(a.1.name.cmp(&b.1.name))
    });

    let tags_list: String = sorted
        .iter()
        .map(|(slug, entry)| {
            let count = entry.posts.len() + entry.stories.len() + entry.decks.len();
            format!(
                "      <li><a href=\"tags/{}.html\">#{}</a> ({})</li>",
                slug, entry.name, count
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    let index_html = templates::render(
        &tmpl.tags_index,
        &[("site_title", site_title), ("root", "."), ("tags", &tags_list)],
    );

    let index_path = out_dir.join("tags.html");
    fs::write(&index_path, index_html)
        .with_context(|| format!("write {:?}", index_path))?;

    Ok(())
}
