use pulldown_cmark::{html, Options, Parser};

/// Render markdown to HTML, with Tufte sidenote (`^[text]`) and margin note
/// (`>[text]`) pre-processing applied before the pulldown-cmark pass.
pub fn render(input: &str) -> String {
    let preprocessed = preprocess(input);
    let options = Options::ENABLE_TABLES | Options::ENABLE_STRIKETHROUGH;
    let parser = Parser::new_ext(&preprocessed, options);
    let mut output = String::new();
    html::push_html(&mut output, parser);
    output
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
pub fn split_slides(input: &str) -> Vec<&str> {
    input.split("\n---\n").collect()
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
        let html = render("Hello ^[a note] world.");
        assert!(html.contains("sidenote-number"));
        assert!(html.contains(r#"<span class="sidenote">a note</span>"#));
        assert!(html.contains("Hello"));
        assert!(html.contains("world."));
    }

    #[test]
    fn marginnote_replaced() {
        let html = render("Hello >[a margin] world.");
        assert!(html.contains("⊕"));
        assert!(html.contains(r#"<span class="marginnote">a margin</span>"#));
    }

    #[test]
    fn note_counter_increments() {
        let html = render("^[first] and ^[second]");
        assert!(html.contains(r#"id="sn-1""#));
        assert!(html.contains(r#"id="sn-2""#));
    }

    #[test]
    fn mixed_note_types_share_counter() {
        let html = render("^[side] and >[margin]");
        assert!(html.contains(r#"id="sn-1""#));
        assert!(html.contains(r#"id="mn-2""#));
    }

    #[test]
    fn nested_brackets_in_note() {
        let html = render("^[see [this] ref]");
        assert!(html.contains(r#"<span class="sidenote">see [this] ref</span>"#));
    }

    #[test]
    fn unclosed_note_left_verbatim() {
        // Should not panic; the raw marker should survive in the output.
        let html = render("^[unclosed");
        assert!(html.contains("^"));
    }

    #[test]
    fn code_span_not_processed() {
        let html = render("code: `^[not a note]`");
        assert!(html.contains("^[not a note]"));
        assert!(!html.contains("sidenote"));
    }

    #[test]
    fn fenced_block_not_processed() {
        let html = render("```\n^[not a note]\n```");
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

}
