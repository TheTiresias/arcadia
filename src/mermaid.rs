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
/// On **light pages** (default), node fills are warm Tufte neutrals and
/// text inside nodes is near-black.
///
/// On **dark pages** (when `bg` parses as a dark hex color), node fills are
/// derived by lightening the background, so the diagram shares the page
/// palette. `primary_text_color` is set to `fg` so text inside nodes remains
/// readable. Non-hex `bg` values fall back to the light theme.
fn tufte_theme(bg: Option<&str>, fg: Option<&str>) -> Theme {
    let background = bg.unwrap_or("#fffff8");
    let text       = fg.unwrap_or("#111111");

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

    // Canvas
    theme.background             = background.to_owned();
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
    fn parse_hex_shorthand() {
        assert_eq!(parse_hex("#fff"), Some((255, 255, 255)));
        assert_eq!(parse_hex("#000"), Some((0, 0, 0)));
    }
}
