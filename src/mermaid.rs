use anyhow::{anyhow, Result};
use mermaid_rs_renderer::{LayoutConfig, RenderOptions, Theme};
use regex::Regex;
use std::sync::OnceLock;

/// Replace ```` ```mermaid ``` ```` fenced blocks in a markdown string with
/// their inline SVG. `bg` and `fg` are the page's background and text colors
/// (from frontmatter); both default to Tufte values when absent.
/// `node_spacing` and `rank_spacing` override the layout defaults when provided
/// (see `mermaid_node_spacing` / `mermaid_rank_spacing` frontmatter fields).
pub fn preprocess(
    input: &str,
    bg: Option<&str>,
    fg: Option<&str>,
    node_spacing: Option<f32>,
    rank_spacing: Option<f32>,
) -> Result<String> {
    static RE: OnceLock<Regex> = OnceLock::new();
    let re = RE.get_or_init(|| {
        Regex::new(r"(?m)^```mermaid\n([\s\S]*?)\n```").unwrap()
    });

    let background = bg.unwrap_or("#fffff8");
    let text       = fg.unwrap_or("#111111");

    // §3b: improved defaults for back-edge routing
    let mut layout = LayoutConfig {
        node_spacing: node_spacing.unwrap_or(80.0),
        ..LayoutConfig::default()
    };
    if let Some(rs) = rank_spacing {
        layout.rank_spacing = rs;
    }
    layout.flowchart.routing.occupancy_weight = 2.5;

    let options = RenderOptions {
        theme: tufte_theme(background, text),
        layout,
    };

    let mut out = String::with_capacity(input.len());
    let mut last_end = 0;

    for cap in re.captures_iter(input) {
        let m = cap.get(0).unwrap();
        let source = cap[1].trim();

        out.push_str(&input[last_end..m.start()]);

        let svg = mermaid_rs_renderer::render_with_options(source, options.clone())
            .map_err(|e| anyhow!("mermaid render failed for diagram {:?}: {}", source, e))?;

        let svg = if !is_dark(background) {
            inject_dark_style(&svg, background, text)
        } else {
            svg
        };

        out.push_str(&svg);
        out.push('\n');

        last_end = m.end();
    }

    out.push_str(&input[last_end..]);
    Ok(out)
}

/// Build a Mermaid theme that sits comfortably inside a Tufte-styled page.
///
/// On **light pages** (default), node fills are warm Tufte neutrals and
/// text inside nodes is near-black.
///
/// On **dark pages** (when `bg` parses as a dark hex color), node fills are
/// derived by lightening the background, so the diagram shares the page
/// palette. `primary_text_color` is set to `fg` so text inside nodes remains
/// readable. Non-hex `bg` values fall back to the light theme.
fn tufte_theme(background: &str, text: &str) -> Theme {

    let (primary, secondary, tertiary, border, node_text) = if is_dark(background) {
        // Dark page: derive node fills by lightening the background colour so
        // they remain in the same palette but provide enough contrast to be
        // distinguishable from the page itself.
        (
            lighten_hex(background, 35),   // primary node fill
            lighten_hex(background, 22),   // secondary fill
            lighten_hex(background, 15),   // tertiary fill
            lighten_hex(background, 65),   // borders
            text.to_owned(),               // text in nodes = page fg (light)
        )
    } else {
        // Light page: warm Tufte neutrals; node text always near-black.
        (
            "#e8e8e0".to_owned(),
            "#f0f0e8".to_owned(),
            "#e0e0d8".to_owned(),
            "#777777".to_owned(),
            "#111111".to_owned(),
        )
    };

    let mut theme = Theme::modern();

    // Transparent background so the page background always shows through,
    // including when Tufte CSS switches to dark mode via prefers-color-scheme.
    // Edge label boxes keep the page color so arrow text stays readable.
    theme.background             = "transparent".to_owned();
    theme.edge_label_background  = background.to_owned();

    // Node fills and borders
    theme.primary_color          = primary.clone();
    theme.secondary_color        = secondary.clone();
    theme.tertiary_color         = tertiary;
    theme.primary_border_color   = border.clone();
    theme.primary_text_color     = node_text;

    // Cluster / subgraph
    theme.cluster_background     = secondary;
    theme.cluster_border         = border.clone();

    // Lines and outer labels track the page text colour
    theme.line_color             = text.to_owned();
    theme.text_color             = text.to_owned();

    // Sequence diagrams inherit the same palette
    theme.sequence_actor_fill    = primary;
    theme.sequence_actor_border  = border.clone();
    theme.sequence_note_fill     = theme.tertiary_color.clone();
    theme.sequence_note_border   = border;
    theme.sequence_actor_line    = text.to_owned();

    // Typography
    theme.font_family = "et-book, Palatino, 'Palatino Linotype', serif".to_owned();

    theme
}

/// Inject a `<style>` block containing `@media (prefers-color-scheme: dark)`
/// overrides into a light-theme SVG, inserted right after the opening `<svg>`
/// tag. The dark palette is derived from Tufte's `#151515` background using the
/// same `lighten_hex` offsets as the light theme.
fn inject_dark_style(svg: &str, light_bg: &str, light_fg: &str) -> String {
    const DARK_BG: &str = "#151515";
    const DARK_FG: &str = "#dddddd";

    let dp = lighten_hex(DARK_BG, 35); // primary   (#383838)
    let ds = lighten_hex(DARK_BG, 22); // secondary (#2b2b2b)
    let dt = lighten_hex(DARK_BG, 15); // tertiary  (#242424)
    let db = lighten_hex(DARK_BG, 65); // border    (#565656)

    let style = format!(
        concat!(
            "<style>@media (prefers-color-scheme:dark){{",
            "[fill=\"#e8e8e0\"]{{fill:{dp}}}",
            "[fill=\"#f0f0e8\"]{{fill:{ds}}}",
            "[fill=\"#e0e0d8\"]{{fill:{dt}}}",
            "[stroke=\"#777777\"]{{stroke:{db}}}",
            "[fill=\"{lfg}\"]{{fill:{DARK_FG}}}",
            "[stroke=\"{lfg}\"]{{stroke:{DARK_FG}}}",
            "[fill=\"{lbg}\"]{{fill:{DARK_BG}}}",
            "}}</style>",
        ),
        dp = dp, ds = ds, dt = dt, db = db,
        lfg = light_fg, lbg = light_bg,
        DARK_FG = DARK_FG, DARK_BG = DARK_BG,
    );

    // Splice the <style> block in right after the `>` that closes <svg ...>.
    if let Some(start) = svg.find("<svg") {
        if let Some(rel) = svg[start..].find('>') {
            let pos = start + rel + 1;
            let mut out = String::with_capacity(svg.len() + style.len());
            out.push_str(&svg[..pos]);
            out.push_str(&style);
            out.push_str(&svg[pos..]);
            return out;
        }
    }
    svg.to_owned()
}

/// Return `true` when `color` is a dark `#RRGGBB` or `#RGB` hex value.
/// Non-hex values (e.g. `hsl(...)`) return `false` (safe light-theme fallback).
fn is_dark(color: &str) -> bool {
    let (r, g, b) = match parse_hex(color) {
        Some(v) => v,
        None => return false,
    };
    // Perceived luminance (BT.601)
    0.299 * r as f32 + 0.587 * g as f32 + 0.114 * (b as f32) < 128.0
}

/// Add `amount` to each RGB channel of a `#RRGGBB` hex color, capped at 255.
/// Returns the original string unchanged if it cannot be parsed.
fn lighten_hex(color: &str, amount: u8) -> String {
    match parse_hex(color) {
        Some((r, g, b)) => format!(
            "#{:02x}{:02x}{:02x}",
            r.saturating_add(amount),
            g.saturating_add(amount),
            b.saturating_add(amount),
        ),
        None => color.to_owned(),
    }
}

/// Parse `#RRGGBB` or `#RGB` into `(r, g, b)` in 0–255 range.
fn parse_hex(color: &str) -> Option<(u8, u8, u8)> {
    let hex = color.strip_prefix('#')?;
    match hex.len() {
        6 => Some((
            u8::from_str_radix(&hex[0..2], 16).ok()?,
            u8::from_str_radix(&hex[2..4], 16).ok()?,
            u8::from_str_radix(&hex[4..6], 16).ok()?,
        )),
        3 => {
            let r = u8::from_str_radix(&hex[0..1], 16).ok()?;
            let g = u8::from_str_radix(&hex[1..2], 16).ok()?;
            let b = u8::from_str_radix(&hex[2..3], 16).ok()?;
            Some((r * 17, g * 17, b * 17))
        }
        _ => None,
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dark_detection_black() {
        assert!(is_dark("#000000"));
    }

    #[test]
    fn dark_detection_dark_navy() {
        assert!(is_dark("#1a1a2e"));
    }

    #[test]
    fn dark_detection_tufte_white() {
        assert!(!is_dark("#fffff8"));
    }

    #[test]
    fn dark_detection_light_gray() {
        assert!(!is_dark("#cccccc"));
    }

    #[test]
    fn dark_detection_non_hex_fallback() {
        assert!(!is_dark("hsl(240, 30%, 15%)"));
    }

    #[test]
    fn lighten_clamps_at_255() {
        assert_eq!(lighten_hex("#ffffff", 10), "#ffffff");
    }

    #[test]
    fn lighten_dark_color() {
        assert_eq!(lighten_hex("#1a1a2e", 35), "#3d3d51");
    }

    #[test]
    fn lighten_non_hex_passthrough() {
        assert_eq!(lighten_hex("hsl(0,0%,0%)", 10), "hsl(0,0%,0%)");
    }

    #[test]
    fn inject_dark_style_light_page() {
        let svg = r##"<svg xmlns="http://www.w3.org/2000/svg"><rect fill="#e8e8e0"/></svg>"##;
        let result = inject_dark_style(svg, "#fffff8", "#111111");
        assert!(result.contains("<style>@media (prefers-color-scheme:dark)"));
        assert!(result.contains("[fill=\"#e8e8e0\"]"));
        assert!(result.starts_with(r#"<svg xmlns="http://www.w3.org/2000/svg"><style>"#));
    }

    #[test]
    fn inject_dark_style_no_svg_tag_passthrough() {
        let bad = "<not-an-svg/>";
        assert_eq!(inject_dark_style(bad, "#fffff8", "#111111"), bad);
    }

    #[test]
    fn parse_hex_shorthand() {
        assert_eq!(parse_hex("#fff"), Some((255, 255, 255)));
        assert_eq!(parse_hex("#000"), Some((0, 0, 0)));
    }
}
