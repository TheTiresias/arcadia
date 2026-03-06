use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use rss::{ChannelBuilder, ItemBuilder};

use crate::content::{DeckMeta, PostMeta, StoryMeta};

pub fn build(out_dir: &Path, posts: &[PostMeta], site_title: &str, base_url: &str) -> Result<()> {
    let base = base_url.trim_end_matches('/');

    let items: Vec<rss::Item> = posts
        .iter()
        .map(|post| {
            let link = format!("{}/posts/{}.html", base, post.slug);
            let mut b = ItemBuilder::default();
            b.title(Some(post.title.clone()));
            b.link(Some(link));
            if !post.date.is_empty() {
                b.pub_date(Some(to_rfc2822(&post.date)));
            }
            if let Some(sub) = &post.subtitle {
                b.description(Some(sub.clone()));
            }
            b.content(Some(post.content_html.clone()));
            b.build()
        })
        .collect();

    let channel = ChannelBuilder::default()
        .title(site_title.to_owned())
        .link(base.to_owned())
        .description(site_title.to_owned())
        .items(items)
        .build();

    fs::write(out_dir.join("feed.xml"), channel.to_string()).context("write feed.xml")?;
    Ok(())
}

pub fn build_fiction(
    out_dir: &Path,
    stories: &[StoryMeta],
    site_title: &str,
    base_url: &str,
) -> Result<()> {
    let base = base_url.trim_end_matches('/');

    let items: Vec<rss::Item> = stories
        .iter()
        .map(|story| {
            let link = format!("{}/fiction/{}/index.html", base, story.slug);
            let mut b = ItemBuilder::default();
            b.title(Some(story.title.clone()));
            b.link(Some(link));
            if !story.description.is_empty() {
                b.description(Some(story.description.clone()));
            }
            b.build()
        })
        .collect();

    let channel = ChannelBuilder::default()
        .title(format!("{} — Fiction", site_title))
        .link(format!("{}/fiction.html", base))
        .description(format!("{} fiction", site_title))
        .items(items)
        .build();

    fs::write(out_dir.join("fiction-feed.xml"), channel.to_string())
        .context("write fiction-feed.xml")?;
    Ok(())
}

pub fn build_decks(
    out_dir: &Path,
    decks: &[DeckMeta],
    site_title: &str,
    base_url: &str,
) -> Result<()> {
    let base = base_url.trim_end_matches('/');

    let items: Vec<rss::Item> = decks
        .iter()
        .map(|deck| {
            let link = format!("{}/decks/{}.html", base, deck.slug);
            let mut b = ItemBuilder::default();
            b.title(Some(deck.title.clone()));
            b.link(Some(link));
            b.build()
        })
        .collect();

    let channel = ChannelBuilder::default()
        .title(format!("{} — Decks", site_title))
        .link(format!("{}/decks.html", base))
        .description(format!("{} slide decks", site_title))
        .items(items)
        .build();

    fs::write(out_dir.join("decks-feed.xml"), channel.to_string())
        .context("write decks-feed.xml")?;
    Ok(())
}

/// Convert `YYYY-MM-DD` to RFC 2822 (`DD Mon YYYY 00:00:00 +0000`).
fn to_rfc2822(date: &str) -> String {
    const MONTHS: [&str; 12] = [
        "Jan", "Feb", "Mar", "Apr", "May", "Jun",
        "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
    ];
    let parts: Vec<&str> = date.splitn(3, '-').collect();
    if parts.len() != 3 {
        return date.to_owned();
    }
    let month = parts[1]
        .parse::<usize>()
        .ok()
        .and_then(|m| MONTHS.get(m.wrapping_sub(1)))
        .copied()
        .unwrap_or("Jan");
    format!("{} {} {} 00:00:00 +0000", parts[2], month, parts[0])
}
