use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;

use anyhow::{Context, Result};

use crate::config::SiteConfig;
use crate::content::{decks, fiction, posts, tags};
use crate::content::{DeckMeta, PostMeta, StoryMeta};
use crate::feeds;
use crate::sitemap;
use crate::templates::{self, Templates};

#[derive(Clone)]
pub struct BuildConfig {
    pub project_dir: PathBuf,
    pub src_dir: PathBuf,
    pub out_dir: PathBuf,
    pub drafts: bool,
    pub site_title: String,
    pub base_url: Option<String>,
}

impl BuildConfig {
    pub fn load(
        project_dir: PathBuf,
        src_dir: PathBuf,
        out_dir: PathBuf,
        drafts: bool,
        site_config: &SiteConfig,
    ) -> Self {
        let site_title = site_config.title.clone().unwrap_or_else(|| "Arcadia".to_owned());
        let base_url = site_config.base_url.clone();
        BuildConfig { project_dir, src_dir, out_dir, drafts, site_title, base_url }
    }
}

#[allow(dead_code)]
pub struct BuildSummary {
    pub post_count: usize,
    pub story_count: usize,
    pub deck_count: usize,
    pub elapsed_ms: u128,
}

pub fn build(config: &BuildConfig) -> Result<BuildSummary> {
    let start = Instant::now();
    let src = config.src_dir.as_path();
    let out = config.out_dir.as_path();
    let drafts = config.drafts;

    // 1. Create output dir
    fs::create_dir_all(out).context("create output dir")?;

    // Load templates (project-local overrides or embedded fallbacks)
    let tmpl = Templates::load(&config.project_dir);

    // 2. Run all three pipelines in parallel
    let (post_result, (fiction_result, deck_result)) = rayon::join(
        || posts::build(src, out, drafts, &tmpl),
        || rayon::join(|| fiction::build(src, out, &tmpl), || decks::build(src, out, &tmpl)),
    );

    let post_metas = post_result.context("posts pipeline")?;
    let story_metas = fiction_result.context("fiction pipeline")?;
    let deck_metas = deck_result.context("decks pipeline")?;

    // 3. Tags pipeline (needs all metas)
    tags::build(out, &post_metas, &story_metas, &deck_metas, &config.site_title, &tmpl)
        .context("tags pipeline")?;

    // 4. Generate index / fiction / decks listing pages
    generate_index(out, &post_metas, &story_metas, &deck_metas, &config.site_title, &tmpl)?;

    // 5. RSS feed and sitemap (only when base_url is configured)
    if let Some(base_url) = &config.base_url {
        feeds::build(out, &post_metas, &config.site_title, base_url)
            .context("feeds pipeline")?;
        feeds::build_fiction(out, &story_metas, &config.site_title, base_url)
            .context("fiction feeds pipeline")?;
        feeds::build_decks(out, &deck_metas, &config.site_title, base_url)
            .context("decks feeds pipeline")?;
        sitemap::build(out, &post_metas, &story_metas, &deck_metas, base_url)
            .context("sitemap pipeline")?;
    }

    // 6. Copy resources/ → dist/resources/
    copy_dir_if_exists(&src.join("resources"), &out.join("resources"), false)?;

    // Write tufte.css from binary or ejected template (always wins over resources/).
    // If the user has run `arcadia eject`, embed/tufte.css is their customised version;
    // otherwise use the copy baked into the binary.
    let resources_out = out.join("resources");
    fs::create_dir_all(&resources_out).context("create resources dir")?;
    let ejected = config.project_dir.join("embed").join("tufte.css");
    let tufte_content = if ejected.exists() {
        fs::read_to_string(&ejected).context("read ejected tufte.css")?
    } else {
        templates::TUFTE_CSS.to_owned()
    };
    fs::write(resources_out.join("tufte.css"), tufte_content).context("write tufte.css")?;

    // 7. Copy images/ → dist/images/
    copy_dir_if_exists(&src.join("images"), &out.join("images"), false)?;

    // 8. Copy assets/ → dist/assets/ (incremental: skip unchanged files)
    copy_dir_if_exists(&src.join("assets"), &out.join("assets"), true)?;

    let elapsed_ms = start.elapsed().as_millis();
    println!(
        "Built {} posts, {} stories, {} decks in {}ms",
        post_metas.len(),
        story_metas.len(),
        deck_metas.len(),
        elapsed_ms
    );

    Ok(BuildSummary {
        post_count: post_metas.len(),
        story_count: story_metas.len(),
        deck_count: deck_metas.len(),
        elapsed_ms,
    })
}

fn generate_index(
    out_dir: &Path,
    posts: &[PostMeta],
    stories: &[StoryMeta],
    decks: &[DeckMeta],
    site_title: &str,
    tmpl: &Templates,
) -> Result<()> {
    // Home index
    let posts_html: String = posts
        .iter()
        .map(|p| {
            format!(
                "      <li><a href=\"posts/{}.html\">{}</a> {}</li>",
                p.slug, p.title, p.date
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    let index_html = templates::render(
        &tmpl.index,
        &[
            ("site_title", site_title),
            ("root", "."),
            ("posts", &posts_html),
            ("has_fiction", if stories.is_empty() { "" } else { "1" }),
            ("has_decks", if decks.is_empty() { "" } else { "1" }),
        ],
    );
    fs::write(out_dir.join("index.html"), index_html).context("write index.html")?;

    // Fiction index
    let stories_html: String = stories
        .iter()
        .map(|s| {
            let ch_label = if s.chapter_count == 1 {
                "1 chapter".to_owned()
            } else {
                format!("{} chapters", s.chapter_count)
            };
            format!(
                "      <li><a href=\"fiction/{}/index.html\">{}</a> — {} ({})</li>",
                s.slug, s.title, s.description, ch_label
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    let fiction_html = templates::render(
        &tmpl.fiction_index,
        &[("site_title", site_title), ("root", "."), ("stories", &stories_html)],
    );
    fs::write(out_dir.join("fiction.html"), fiction_html).context("write fiction.html")?;

    // Decks index
    let decks_html: String = decks
        .iter()
        .map(|d| {
            format!(
                "      <li><a href=\"decks/{}.html\">{}</a></li>",
                d.slug, d.title
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    let decks_page = templates::render(
        &tmpl.decks_index,
        &[("site_title", site_title), ("root", "."), ("decks", &decks_html)],
    );
    fs::write(out_dir.join("decks.html"), decks_page).context("write decks.html")?;

    // 404 page
    let not_found_html = templates::render(
        &tmpl.not_found,
        &[("title", site_title), ("root", ".")],
    );
    fs::write(out_dir.join("404.html"), not_found_html).context("write 404.html")?;

    Ok(())
}

fn copy_dir_if_exists(src: &Path, dst: &Path, incremental: bool) -> Result<()> {
    if !src.exists() {
        return Ok(());
    }
    copy_dir(src, dst, incremental)
}

fn copy_dir(src: &Path, dst: &Path, incremental: bool) -> Result<()> {
    fs::create_dir_all(dst).with_context(|| format!("create dir {:?}", dst))?;
    for entry in fs::read_dir(src).with_context(|| format!("read dir {:?}", src))? {
        let entry = entry?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());
        if src_path.is_dir() {
            copy_dir(&src_path, &dst_path, incremental)?;
        } else {
            if incremental && dst_path.exists() {
                let src_mtime = fs::metadata(&src_path)?.modified()?;
                let dst_mtime = fs::metadata(&dst_path)?.modified()?;
                if dst_mtime >= src_mtime {
                    continue;
                }
            }
            fs::copy(&src_path, &dst_path)
                .with_context(|| format!("copy {:?} → {:?}", src_path, dst_path))?;
        }
    }
    Ok(())
}
