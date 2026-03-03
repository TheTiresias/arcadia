use std::fs;
use std::path::Path;

use anyhow::{Context, Result};

use super::{body_style, render_tag_pills, str_field, tags_field, DeckMeta};
use crate::{frontmatter, markdown, templates};

pub fn build(src_dir: &Path, out_dir: &Path) -> Result<Vec<DeckMeta>> {
    let decks_src = src_dir.join("decks");
    let decks_out = out_dir.join("decks");
    fs::create_dir_all(&decks_out).context("create dist/decks")?;

    if !decks_src.exists() {
        return Ok(Vec::new());
    }

    let mut metas: Vec<DeckMeta> = Vec::new();

    for entry in fs::read_dir(&decks_src).context("read src/decks")? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("md") {
            continue;
        }

        let source =
            fs::read_to_string(&path).with_context(|| format!("read {:?}", path))?;
        let (meta, body) = frontmatter::parse(&source)
            .with_context(|| format!("parse frontmatter in {:?}", path))?;

        let title = str_field(&meta, "title").unwrap_or_else(|| "Untitled".to_owned());
        let tags = tags_field(&meta);
        let bstyle = body_style(&meta);

        let slug = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("deck")
            .to_owned();

        let slides_html: String = markdown::split_slides(body)
            .iter()
            .map(|slide| {
                format!("<div class=\"slide\">{}</div>", markdown::render(slide))
            })
            .collect::<Vec<_>>()
            .join("\n");

        let tags_html = render_tag_pills(&tags, "..", true);

        let html = templates::render(
            templates::SLIDE_DECK,
            &[
                ("title", &title),
                ("root", ".."),
                ("body_style", &bstyle),
                ("tags", &tags_html),
                ("slides", &slides_html),
            ],
        );

        let out_path = decks_out.join(format!("{}.html", slug));
        fs::write(&out_path, html).with_context(|| format!("write {:?}", out_path))?;

        metas.push(DeckMeta { title, slug, tags });
    }

    Ok(metas)
}
