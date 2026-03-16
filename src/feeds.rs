use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use chrono::NaiveDate;
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

    write_feed(site_title, site_title, base, &out_dir.join("feed.xml"), items)
        .context("write feed.xml")
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

    write_feed(
        &format!("{} — Fiction", site_title),
        &format!("{} fiction", site_title),
        &format!("{}/fiction.html", base),
        &out_dir.join("fiction-feed.xml"),
        items,
    )
    .context("write fiction-feed.xml")
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

    write_feed(
        &format!("{} — Decks", site_title),
        &format!("{} slide decks", site_title),
        &format!("{}/decks.html", base),
        &out_dir.join("decks-feed.xml"),
        items,
    )
    .context("write decks-feed.xml")
}

fn write_feed(
    title: &str,
    description: &str,
    link: &str,
    output_path: &Path,
    items: Vec<rss::Item>,
) -> Result<()> {
    let channel = ChannelBuilder::default()
        .title(title.to_owned())
        .link(link.to_owned())
        .description(description.to_owned())
        .items(items)
        .build();
    fs::write(output_path, channel.to_string())
        .with_context(|| format!("write {:?}", output_path))
}

/// Convert `YYYY-MM-DD` to RFC 2822. Returns an empty string on invalid input.
fn to_rfc2822(date: &str) -> String {
    NaiveDate::parse_from_str(date, "%Y-%m-%d")
        .ok()
        .and_then(|d| d.and_hms_opt(0, 0, 0))
        .map(|dt| dt.and_utc().to_rfc2822())
        .unwrap_or_default()
}
