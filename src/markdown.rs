use anyhow::Result;
use pulldown_cmark::{html, CodeBlockKind, Event, Options, Parser, Tag, TagEnd};
use regex::Regex;
use std::sync::OnceLock;

use crate::mermaid;

/// Render markdown to HTML.
///
/// `bg` and `fg` are the page's background and text colors (from frontmatter);
/// they are forwarded to the Mermaid renderer so diagrams match the page.
/// `node_spacing` and `rank_spacing` override Mermaid layout defaults when provided.
/// Pass `None` for any argument not set in frontmatter.
///
/// Passes through three stages:
/// 1. Mermaid fenced blocks (```` ```mermaid ````) are replaced with inline SVG.
/// 2. Tufte sidenote (`^[text]`) and margin note (`>[text]`) markers are expanded.
/// 3. pulldown-cmark renders the result to HTML.
pub fn render(
    input: &str,
    bg: Option<&str>,
    fg: Option<&str>,
    node_spacing: Option<f32>,
    rank_spacing: Option<f32>,
) -> Result<String> {
    let after_mermaid = mermaid::preprocess(input, bg, fg, node_spacing, rank_spacing)?;
    let preprocessed = preprocess(&after_mermaid);
    let options =
        Options::ENABLE_TABLES | Options::ENABLE_STRIKETHROUGH | Options::ENABLE_FOOTNOTES;
    let parser = Parser::new_ext(&preprocessed, options);

    let dark_bg = bg.map(mermaid::is_dark).unwrap_or(false);

    // B3: intercept fenced code blocks for syntax highlighting
    let mut events: Vec<Event> = Vec::new();
    let mut in_code = false;
    let mut code_lang = String::new();
    let mut code_buf = String::new();
    for event in parser {
        match event {
            Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(ref lang))) => {
                in_code = true;
                code_lang = lang.to_string();
                code_buf.clear();
            }
            Event::Text(ref text) if in_code => {
                code_buf.push_str(text);
            }
            Event::End(TagEnd::CodeBlock) if in_code => {
                in_code = false;
                let highlighted = syntax_highlight(&code_lang, &code_buf, dark_bg);
                events.push(Event::Html(highlighted.into()));
            }
            other if in_code => drop(other),
            other => events.push(other),
        }
    }

    let mut output = String::new();
    html::push_html(&mut output, events.into_iter());

    // B2: add id + anchor link to every heading
    let output = add_heading_anchors(&output);

    // B1: wrap images inside sidenotes/marginnotes with CSS-only lightbox
    let output = add_sidenote_lightboxes(&output);

    Ok(output)
}

/// Wrap a rendered HTML string in `<section>` blocks, splitting on `<hr />`.
/// Used for posts where `---` in the source creates section breaks.
pub fn section_wrap(html: &str) -> String {
    html.split("<hr />")
        .map(|part| format!("<section>\n{}\n</section>", part.trim()))
        .collect::<Vec<_>>()
        .join("\n")
}

/// Split raw markdown on `\n---\n` for use in slide decks.
/// Returns raw markdown chunks (not yet rendered); each chunk is one slide.
///
/// `---` lines inside fenced code blocks (``` or ~~~) are ignored so that
/// YAML and markdown examples inside code blocks don't break slide boundaries.
pub fn split_slides(input: &str) -> Vec<&str> {
    let mut slides = Vec::new();
    let mut in_fence = false;
    let mut slide_start = 0;
    let mut pos = 0;

    for line in input.split_inclusive('\n') {
        let t = line.trim_end_matches('\n').trim_end_matches('\r');
        if t.starts_with("```") || t.starts_with("~~~") {
            in_fence = !in_fence;
        } else if !in_fence && t == "---" && pos > 0 && line.ends_with('\n') {
            slides.push(&input[slide_start..pos - 1]);
            slide_start = pos + line.len();
        }
        pos += line.len();
    }

    slides.push(&input[slide_start..]);
    slides
}

// ── Syntax highlighting (B3) ───────────────────────────────────────────────────

fn syntax_highlight(lang: &str, code: &str, dark: bool) -> String {
    use syntect::easy::HighlightLines;
    use syntect::highlighting::ThemeSet;
    use syntect::html::{styled_line_to_highlighted_html, IncludeBackground};
    use syntect::parsing::SyntaxSet;
    use syntect::util::LinesWithEndings;

    static SS: OnceLock<SyntaxSet> = OnceLock::new();
    static TS: OnceLock<ThemeSet> = OnceLock::new();

    let ss = SS.get_or_init(SyntaxSet::load_defaults_newlines);
    let ts = TS.get_or_init(ThemeSet::load_defaults);

    let syntax = ss
        .find_syntax_by_token(lang)
        .unwrap_or_else(|| ss.find_syntax_plain_text());
    let theme_name = if dark { "base16-ocean.dark" } else { "InspiredGitHub" };
    let theme = ts
        .themes
        .get(theme_name)
        .or_else(|| ts.themes.values().next())
        .expect("syntect default themes are non-empty");

    let mut h = HighlightLines::new(syntax, theme);
    let mut out = String::from("<pre><code>");
    for line in LinesWithEndings::from(code) {
        match h.highlight_line(line, ss) {
            Ok(ranges) => match styled_line_to_highlighted_html(&ranges, IncludeBackground::No) {
                Ok(html) => out.push_str(&html),
                Err(_) => out.push_str(&html_escape::encode_text(line)),
            },
            Err(_) => out.push_str(&html_escape::encode_text(line)),
        }
    }
    out.push_str("</code></pre>");
    out
}

// ── Heading anchors (B2) ───────────────────────────────────────────────────────

fn add_heading_anchors(html: &str) -> String {
    static HEADING_RE: OnceLock<Regex> = OnceLock::new();
    static STRIP_TAGS_RE: OnceLock<Regex> = OnceLock::new();

    let heading_re =
        HEADING_RE.get_or_init(|| Regex::new(r"(?s)<(h[1-6])>(.*?)</h[1-6]>").unwrap());
    let strip_re = STRIP_TAGS_RE.get_or_init(|| Regex::new(r"<[^>]+>").unwrap());

    heading_re
        .replace_all(html, |caps: &regex::Captures| {
            let tag = &caps[1];
            let content = &caps[2];
            let text = strip_re.replace_all(content, "");
            let id = slug::slugify(&*text);
            format!(
                r##"<{tag} id="{id}">{content}<a href="#{id}" class="heading-anchor">#</a></{tag}>"##
            )
        })
        .into_owned()
}

// ── Sidenote image lightboxes (B1) ────────────────────────────────────────────

/// Wrap `<img>` tags inside `.sidenote` and `.marginnote` spans with a
/// CSS-only `:target` lightbox.
///
/// For each image found, the img is replaced with a trigger link and a
/// corresponding overlay span is appended immediately after the closing
/// `</span>` of the sidenote.
fn add_sidenote_lightboxes(html: &str) -> String {
    static SPAN_RE: OnceLock<Regex> = OnceLock::new();
    static IMG_RE: OnceLock<Regex> = OnceLock::new();

    let span_re = SPAN_RE.get_or_init(|| {
        Regex::new(r#"(?s)<span class="(sidenote|marginnote)">(.*?)</span>"#).unwrap()
    });
    let img_re = IMG_RE.get_or_init(|| Regex::new(r#"<img([^>]*)>"#).unwrap());

    let mut counter = 0usize;
    let mut result = String::with_capacity(html.len() + 512);
    let mut last = 0;

    for span_cap in span_re.captures_iter(html) {
        let span_match = span_cap.get(0).unwrap();
        let class = &span_cap[1];
        let inner = &span_cap[2];

        if !img_re.is_match(inner) {
            continue;
        }

        result.push_str(&html[last..span_match.start()]);
        result.push_str(&format!(r#"<span class="{class}">"#));

        let mut overlays = String::new();
        let mut inner_last = 0;

        for img_cap in img_re.captures_iter(inner) {
            let img_match = img_cap.get(0).unwrap();
            counter += 1;
            let id = format!("lb-img-{counter}");
            let img_tag = img_match.as_str();

            result.push_str(&inner[inner_last..img_match.start()]);
            result.push_str(&format!(
                "<a href=\"#{}\" class=\"lightbox-trigger\">{}</a>",
                id, img_tag
            ));
            overlays.push_str(&format!(
                "<span id=\"{}\" class=\"lightbox\"><a href=\"#_\" class=\"lightbox-close\"></a>{}</span>",
                id, img_tag
            ));

            inner_last = img_match.end();
        }

        result.push_str(&inner[inner_last..]);
        result.push_str("</span>");
        result.push_str(&overlays);
        last = span_match.end();
    }

    result.push_str(&html[last..]);
    result
}

// ── Pre-processing ────────────────────────────────────────────────────────────

/// Replace `^[...]` (sidenote) and `>[...]` (margin note) markers in the
/// source markdown with their Tufte HTML equivalents.
///
/// Code spans (`` `...` ``) and fenced code blocks (` ``` `) are passed
/// through verbatim so markers inside code are never replaced.
///
/// Note: all marker bytes (`^`, `>`, `` ` ``, `[`, `]`) are single-byte ASCII
/// and can never appear as UTF-8 continuation bytes, so byte-level scanning
/// is safe throughout.
fn preprocess(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut note_count = 0usize;
    let bytes = input.as_bytes();
    let len = bytes.len();
    let mut pos = 0;
    let mut copy_start = 0;

    while pos < len {
        let b = bytes[pos];

        // Fenced code block: ``` … ```.  Consume through the closing fence so
        // markers inside are never substituted.
        if b == b'`' && pos + 2 < len && bytes[pos + 1] == b'`' && bytes[pos + 2] == b'`' {
            out.push_str(&input[copy_start..pos]);
            let fence = &input[pos..];
            // Search for the first \n``` after the opening fence.
            let consumed = fence[3..].find("\n```").map_or(fence.len(), |i| i + 7);
            out.push_str(&fence[..consumed]);
            pos += consumed;
            copy_start = pos;
            continue;
        }

        // Code span: `…`.  Consume through the closing backtick.
        if b == b'`' {
            out.push_str(&input[copy_start..pos]);
            let span = &input[pos..];
            let consumed = span[1..].find('`').map_or(span.len(), |i| i + 2);
            out.push_str(&span[..consumed]);
            pos += consumed;
            copy_start = pos;
            continue;
        }

        // Sidenote ^[…] or margin note >[…].
        if (b == b'^' || b == b'>') && pos + 1 < len && bytes[pos + 1] == b'[' {
            let content_start = pos + 2;
            if let Some(close) = find_closing_bracket(&input[content_start..]) {
                out.push_str(&input[copy_start..pos]);
                let content = &input[content_start..content_start + close];
                note_count += 1;
                if b == b'^' {
                    out.push_str(&sidenote_html(note_count, content));
                } else {
                    out.push_str(&marginnote_html(note_count, content));
                }
                pos = content_start + close + 1;
                copy_start = pos;
                continue;
            }
        }

        pos += 1;
    }

    if copy_start < len {
        out.push_str(&input[copy_start..]);
    }

    out
}

/// Return the byte index of the `]` that closes the bracket sequence starting
/// at the beginning of `s` (the opening `[` has already been consumed).
/// Handles nested brackets. Returns `None` if no matching `]` is found.
fn find_closing_bracket(s: &str) -> Option<usize> {
    let mut depth = 1usize;
    for (i, ch) in s.char_indices() {
        match ch {
            '[' => depth += 1,
            ']' => {
                depth -= 1;
                if depth == 0 {
                    return Some(i);
                }
            }
            _ => {}
        }
    }
    None
}

fn sidenote_html(id: usize, content: &str) -> String {
    format!(
        r#"<label for="sn-{id}" class="margin-toggle sidenote-number"></label><input type="checkbox" id="sn-{id}" class="margin-toggle"/><span class="sidenote">{content}</span>"#
    )
}

fn marginnote_html(id: usize, content: &str) -> String {
    format!(
        r#"<label for="mn-{id}" class="margin-toggle">⊕</label><input type="checkbox" id="mn-{id}" class="margin-toggle"/><span class="marginnote">{content}</span>"#
    )
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sidenote_replaced() {
        let html = render("Hello ^[a note] world.", None, None, None, None).unwrap();
        assert!(html.contains("sidenote-number"));
        assert!(html.contains(r#"<span class="sidenote">a note</span>"#));
        assert!(html.contains("Hello"));
        assert!(html.contains("world."));
    }

    #[test]
    fn marginnote_replaced() {
        let html = render("Hello >[a margin] world.", None, None, None, None).unwrap();
        assert!(html.contains("⊕"));
        assert!(html.contains(r#"<span class="marginnote">a margin</span>"#));
    }

    #[test]
    fn note_counter_increments() {
        let html = render("^[first] and ^[second]", None, None, None, None).unwrap();
        assert!(html.contains(r#"id="sn-1""#));
        assert!(html.contains(r#"id="sn-2""#));
    }

    #[test]
    fn mixed_note_types_share_counter() {
        let html = render("^[side] and >[margin]", None, None, None, None).unwrap();
        assert!(html.contains(r#"id="sn-1""#));
        assert!(html.contains(r#"id="mn-2""#));
    }

    #[test]
    fn nested_brackets_in_note() {
        let html = render("^[see [this] ref]", None, None, None, None).unwrap();
        assert!(html.contains(r#"<span class="sidenote">see [this] ref</span>"#));
    }

    #[test]
    fn unclosed_note_left_verbatim() {
        // Should not panic; the raw marker should survive in the output.
        let html = render("^[unclosed", None, None, None, None).unwrap();
        assert!(html.contains("^"));
    }

    #[test]
    fn code_span_not_processed() {
        let html = render("code: `^[not a note]`", None, None, None, None).unwrap();
        assert!(html.contains("^[not a note]"));
        assert!(!html.contains("sidenote"));
    }

    #[test]
    fn fenced_block_not_processed() {
        let html = render("```\n^[not a note]\n```", None, None, None, None).unwrap();
        assert!(html.contains("^[not a note]"));
        assert!(!html.contains("sidenote"));
    }

    #[test]
    fn section_wrap_splits_on_hr() {
        let html = "<p>one</p>\n<hr />\n<p>two</p>";
        let wrapped = section_wrap(html);
        assert_eq!(wrapped.matches("<section>").count(), 2);
        assert_eq!(wrapped.matches("</section>").count(), 2);
    }

    #[test]
    fn section_wrap_single_section() {
        let html = "<p>no breaks</p>";
        let wrapped = section_wrap(html);
        assert_eq!(wrapped.matches("<section>").count(), 1);
    }

    #[test]
    fn split_slides_three_slides() {
        let src = "slide one\n---\nslide two\n---\nslide three";
        let slides = split_slides(src);
        assert_eq!(slides.len(), 3);
        assert_eq!(slides[0], "slide one");
        assert_eq!(slides[1], "slide two");
        assert_eq!(slides[2], "slide three");
    }

    #[test]
    fn split_slides_single_slide() {
        let slides = split_slides("only one slide");
        assert_eq!(slides.len(), 1);
    }

    #[test]
    fn split_slides_ignores_dashes_in_fenced_block() {
        let src = "slide one\n```yaml\n---\nkey: value\n---\n```\n---\nslide two";
        let slides = split_slides(src);
        assert_eq!(slides.len(), 2);
        assert!(slides[0].contains("```yaml"));
        assert_eq!(slides[1], "slide two");
    }

    #[test]
    fn split_slides_ignores_dashes_in_tilde_fence() {
        let src = "slide one\n~~~\n---\n~~~\n---\nslide two";
        let slides = split_slides(src);
        assert_eq!(slides.len(), 2);
        assert_eq!(slides[1], "slide two");
    }
}
