use std::fs;
use std::path::Path;

use anyhow::{Context, Result};

use super::{render_tag_pills, str_field, tags_field, PostMeta};
use crate::templates::{self, Templates};
use crate::{frontmatter, markdown};

pub fn build(src_dir: &Path, out_dir: &Path, drafts: bool, tmpl: &Templates) -> Result<Vec<PostMeta>> {
    let posts_src = src_dir.join("posts");
    let posts_out = out_dir.join("posts");
    fs::create_dir_all(&posts_out).context("create dist/posts")?;

    if !posts_src.exists() {
        return Ok(Vec::new());
    }

    let mut metas: Vec<PostMeta> = Vec::new();

    for entry in fs::read_dir(&posts_src).context("read src/posts")? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("md") {
            continue;
        }

        let source =
            fs::read_to_string(&path).with_context(|| format!("read {:?}", path))?;
        let (meta, body) = frontmatter::parse(&source)
            .with_context(|| format!("parse frontmatter in {:?}", path))?;

        if meta.get("draft").and_then(|v| v.as_bool()).unwrap_or(false) && !drafts {
            continue;
        }

        let title = str_field(&meta, "title").unwrap_or_else(|| "Untitled".to_owned());
        let date = str_field(&meta, "date").unwrap_or_default();
        let subtitle = str_field(&meta, "subtitle");
        let tags = tags_field(&meta);

        let slug = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("post")
            .to_owned();

        let content = markdown::section_wrap(
            &markdown::render(body).with_context(|| format!("render {:?}", path))?,
        );

        let subtitle_html = subtitle
            .as_deref()
            .map(|s| format!("<p class=\"subtitle\">{}</p>", s))
            .unwrap_or_default();
        let date_html = if date.is_empty() {
            String::new()
        } else {
            format!("<p class=\"date\">{}</p>", date)
        };
        let tags_html = render_tag_pills(&tags, "..", false);

        let html = templates::render(
            &tmpl.post,
            &[
                ("title", &title),
                ("root", ".."),
                ("subtitle", &subtitle_html),
                ("date", &date_html),
                ("tags", &tags_html),
                ("content", &content),
            ],
        );

        let out_path = posts_out.join(format!("{}.html", slug));
        fs::write(&out_path, html).with_context(|| format!("write {:?}", out_path))?;

        metas.push(PostMeta { title, slug, date, subtitle, tags });
    }

    // Sort date descending (ISO string lexicographic)
    metas.sort_by(|a, b| b.date.cmp(&a.date));
    Ok(metas)
}
