use std::fs;
use std::path::Path;

use anyhow::{Context, Result};

use super::{body_style, render_tag_pills, str_field, tags_field, StoryMeta};
use crate::{frontmatter, markdown, templates};

struct ChapterInfo {
    slug: String,
    title: String,
    subtitle: Option<String>,
    order: i64,
    body: String,
}

pub fn build(src_dir: &Path, out_dir: &Path) -> Result<Vec<StoryMeta>> {
    let fiction_src = src_dir.join("fiction");
    let fiction_out = out_dir.join("fiction");

    if !fiction_src.exists() {
        return Ok(Vec::new());
    }

    let mut story_metas: Vec<StoryMeta> = Vec::new();

    for entry in fs::read_dir(&fiction_src).context("read src/fiction")? {
        let entry = entry?;
        let story_path = entry.path();
        if !story_path.is_dir() {
            continue;
        }

        let story_slug = story_path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("story")
            .to_owned();

        let story_md_path = story_path.join("story.md");
        if !story_md_path.exists() {
            continue;
        }

        let story_src = fs::read_to_string(&story_md_path)
            .with_context(|| format!("read {:?}", story_md_path))?;
        let (story_meta, _) = frontmatter::parse(&story_src)
            .with_context(|| format!("parse frontmatter in {:?}", story_md_path))?;

        let story_title =
            str_field(&story_meta, "title").unwrap_or_else(|| "Untitled".to_owned());
        let description = str_field(&story_meta, "description").unwrap_or_default();
        let tags = tags_field(&story_meta);
        let bstyle = body_style(&story_meta);

        // Parse chapter files
        let mut chapters: Vec<ChapterInfo> = Vec::new();
        for ch_entry in fs::read_dir(&story_path).context("read story dir")? {
            let ch_entry = ch_entry?;
            let ch_path = ch_entry.path();
            if ch_path.extension().and_then(|e| e.to_str()) != Some("md") {
                continue;
            }
            if ch_path.file_name().and_then(|n| n.to_str()) == Some("story.md") {
                continue;
            }

            let ch_src = fs::read_to_string(&ch_path)
                .with_context(|| format!("read {:?}", ch_path))?;
            let (ch_meta, ch_body) = frontmatter::parse(&ch_src)
                .with_context(|| format!("parse frontmatter in {:?}", ch_path))?;

            let ch_title =
                str_field(&ch_meta, "title").unwrap_or_else(|| "Chapter".to_owned());
            let ch_subtitle = str_field(&ch_meta, "subtitle");
            let order = ch_meta.get("order").and_then(|v| v.as_i64()).unwrap_or(0);
            let ch_slug = ch_path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("chapter")
                .to_owned();

            chapters.push(ChapterInfo {
                slug: ch_slug,
                title: ch_title,
                subtitle: ch_subtitle,
                order,
                body: ch_body.to_owned(),
            });
        }
        chapters.sort_by_key(|c| c.order);

        let story_out = fiction_out.join(&story_slug);
        fs::create_dir_all(&story_out)
            .with_context(|| format!("create dir {:?}", story_out))?;

        let n = chapters.len();
        for (i, ch) in chapters.iter().enumerate() {
            let prev_nav = if i > 0 {
                format!(
                    r#"<a href="{}.html">← {}</a>"#,
                    chapters[i - 1].slug,
                    chapters[i - 1].title
                )
            } else {
                r#"<span class="inactive">←</span>"#.to_owned()
            };
            let next_nav = if i + 1 < n {
                format!(
                    r#"<a href="{}.html">{} →</a>"#,
                    chapters[i + 1].slug,
                    chapters[i + 1].title
                )
            } else {
                r#"<span class="inactive">→</span>"#.to_owned()
            };
            let nav = format!(
                r#"<nav class="chapter-nav">{} — <a href="index.html">Contents</a> — {}</nav>"#,
                prev_nav, next_nav
            );

            let subtitle_html = ch
                .subtitle
                .as_deref()
                .map(|s| format!("<p class=\"subtitle\">{}</p>", s))
                .unwrap_or_default();
            let content = markdown::section_wrap(&markdown::render(&ch.body));

            let html = templates::render(
                templates::CHAPTER,
                &[
                    ("title", &ch.title),
                    ("story_title", &story_title),
                    ("root", "../.."),
                    ("body_style", &bstyle),
                    ("subtitle", &subtitle_html),
                    ("nav", &nav),
                    ("content", &content),
                ],
            );

            let out_path = story_out.join(format!("{}.html", ch.slug));
            fs::write(&out_path, html)
                .with_context(|| format!("write {:?}", out_path))?;
        }

        // Build ToC
        let chapters_html: String = chapters
            .iter()
            .map(|ch| {
                format!(
                    r#"      <li><a href="{}.html">{}</a></li>"#,
                    ch.slug, ch.title
                )
            })
            .collect::<Vec<_>>()
            .join("\n");

        let tags_html = render_tag_pills(&tags, "../..", false);
        let desc_html = if description.is_empty() {
            String::new()
        } else {
            format!("<p>{}</p>", description)
        };

        let toc_html = templates::render(
            templates::STORY_TOC,
            &[
                ("title", &story_title),
                ("root", "../.."),
                ("body_style", &bstyle),
                ("description", &desc_html),
                ("tags", &tags_html),
                ("chapters", &chapters_html),
            ],
        );

        let toc_path = story_out.join("index.html");
        fs::write(&toc_path, toc_html)
            .with_context(|| format!("write {:?}", toc_path))?;

        story_metas.push(StoryMeta {
            title: story_title,
            slug: story_slug,
            description,
            chapter_count: n,
            tags,
        });
    }

    Ok(story_metas)
}
