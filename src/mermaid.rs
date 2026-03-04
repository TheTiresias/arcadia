use anyhow::{anyhow, Result};
use mermaid_rs_renderer::{LayoutConfig, RenderOptions, Theme};
use regex::Regex;
use std::sync::OnceLock;

/// Replace ```` ```mermaid ``` ```` fenced blocks in a markdown string with
/// their inline SVG. `bg` and `fg` are the page's background and text colors
/// (from frontmatter); both default to Tufte values when absent.
pub fn preprocess(input: &str, bg: Option<&str>, fg: Option<&str>) -> Result<String> {
    static RE: OnceLock<Regex> = OnceLock::new();
    let re = RE.get_or_init(|| {
        Regex::new(r"(?m)^```mermaid\n([\s\S]*?)\n```").unwrap()
    });

    let options = RenderOptions {
        theme: tufte_theme(bg, fg),
        layout: LayoutConfig::default(),
    };

    let mut out = String::with_capacity(input.len());
    let mut last_end = 0;

    for cap in re.captures_iter(input) {
        let m = cap.get(0).unwrap();
        let source = cap[1].trim();

        out.push_str(&input[last_end..m.start()]);

        let svg = mermaid_rs_renderer::render_with_options(source, options.clone())
            .map_err(|e| anyhow!("mermaid render failed for diagram {:?}: {}", source, e))?;

        out.push_str(&svg);
        out.push('\n');

        last_end = m.end();
    }

    out.push_str(&input[last_end..]);
    Ok(out)
}

/// Build a Mermaid theme that sits comfortably inside a Tufte-styled page.
///
/// - Background and edge/text colors follow the page's bg/fg if supplied,
///   otherwise fall back to Tufte's warm off-white (`#fffff8`) and near-black.
/// - Node fills are kept as warm neutrals so they are always readable
///   regardless of the page background.
/// - `primary_text_color` (text inside nodes) is always dark since it sits
///   on a light fill.
/// - Font family matches Tufte CSS.
fn tufte_theme(bg: Option<&str>, fg: Option<&str>) -> Theme {
    let background = bg.unwrap_or("#fffff8").to_owned();
    let text       = fg.unwrap_or("#111111").to_owned();

    let mut theme = Theme::modern();

    // Page-level colors
    theme.background          = background.clone();
    theme.edge_label_background = background;

    // Lines and general labels track the page text color
    theme.line_color          = text.clone();
    theme.text_color          = text;

    // Node fills — warm Tufte neutrals, light so they always read well
    theme.primary_color       = "#e8e8e0".to_owned();
    theme.secondary_color     = "#f0f0e8".to_owned();
    theme.tertiary_color      = "#e0e0d8".to_owned();
    theme.primary_border_color = "#777777".to_owned();

    // Text inside nodes is always dark (sits on a light fill)
    theme.primary_text_color  = "#111111".to_owned();

    // Cluster/subgraph styling
    theme.cluster_background  = "#f0f0e8".to_owned();
    theme.cluster_border      = "#aaaaaa".to_owned();

    // Typography
    theme.font_family = "et-book, Palatino, 'Palatino Linotype', serif".to_owned();

    theme
}
