pub mod decks;
pub mod fiction;
pub mod posts;
pub mod tags;

use serde_yaml::Value;

#[derive(Clone)]
pub struct PostMeta {
    pub title: String,
    pub slug: String,
    pub date: String,
    pub subtitle: Option<String>,
    pub tags: Vec<String>,
}

#[derive(Clone)]
pub struct StoryMeta {
    pub title: String,
    pub slug: String,
    pub description: String,
    pub chapter_count: usize,
    pub tags: Vec<String>,
}

#[derive(Clone)]
pub struct DeckMeta {
    pub title: String,
    pub slug: String,
    pub tags: Vec<String>,
}

pub(crate) fn str_field(v: &Value, key: &str) -> Option<String> {
    v.get(key)?.as_str().map(|s| s.to_owned())
}

pub(crate) fn tags_field(v: &Value) -> Vec<String> {
    v.get("tags")
        .and_then(|t| t.as_sequence())
        .map(|seq| {
            seq.iter()
                .filter_map(|item| item.as_str())
                .map(|s| s.to_owned())
                .collect()
        })
        .unwrap_or_default()
}

pub(crate) fn body_style(v: &Value) -> String {
    let bg = str_field(v, "background_color");
    let fg = str_field(v, "font_color");
    match (bg, fg) {
        (Some(bg), Some(fg)) => {
            format!(r#" style="background-color: {}; color: {}""#, bg, fg)
        }
        (Some(bg), None) => format!(r#" style="background-color: {}""#, bg),
        (None, Some(fg)) => format!(r#" style="color: {}""#, fg),
        (None, None) => String::new(),
    }
}

pub(crate) fn tag_slug(tag: &str) -> String {
    tag.to_lowercase()
        .replace(' ', "-")
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '-')
        .collect()
}

/// Render tag pills as either an inline span or a block paragraph.
/// `inline = true` → prefixed with `&nbsp;· ` for use in headers.
/// `inline = false` → wrapped in `<p class="tags">`.
pub(crate) fn render_tag_pills(tags: &[String], root: &str, inline: bool) -> String {
    if tags.is_empty() {
        return String::new();
    }
    let pills: String = tags
        .iter()
        .map(|tag| {
            let slug = tag_slug(tag);
            format!(r#"<a href="{}/tags/{}.html">#{}</a>"#, root, slug, tag)
        })
        .collect::<Vec<_>>()
        .join(" ");
    if inline {
        format!(r#"&nbsp;· {}"#, pills)
    } else {
        format!(r#"<p class="tags">{}</p>"#, pills)
    }
}
