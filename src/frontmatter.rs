use serde_yaml::Value;

/// Parse a `---`-delimited YAML frontmatter block from the beginning of `input`.
///
/// Returns `(metadata, body)` where `body` is the remainder of the input after
/// the closing delimiter. Both slices borrow from `input` — no copies made.
///
/// If the input does not start with `---\n`, or has no valid closing delimiter,
/// an empty mapping is returned and the entire input is treated as the body.
pub fn parse(input: &str) -> Result<(Value, &str), serde_yaml::Error> {
    let Some(after_open) = input.strip_prefix("---\n") else {
        return Ok((Value::Mapping(Default::default()), input));
    };

    let Some((yaml_end, body_start)) = find_close(after_open) else {
        return Ok((Value::Mapping(Default::default()), input));
    };

    let yaml_str = &after_open[..yaml_end];
    let body = &after_open[body_start..];

    let value = serde_yaml::from_str(yaml_str)?;
    Ok((value, body))
}

/// Find the closing `---` delimiter in `s` (the content after the opening `---\n`).
///
/// Returns `(yaml_end, body_start)` where `yaml_end` is the exclusive end of
/// the YAML text and `body_start` is where the body begins (past the delimiter
/// and its trailing newline).
///
/// The closing delimiter must be exactly `---` on its own line — `---more` is
/// not matched, preventing accidental early termination.
fn find_close(s: &str) -> Option<(usize, usize)> {
    // Case 1: closing delimiter at position 0 (empty frontmatter).
    if let Some(after) = s.strip_prefix("---") {
        if after.is_empty() {
            return Some((0, 3));
        }
        if after.starts_with('\n') {
            return Some((0, 4));
        }
    }

    // Case 2: closing delimiter after some YAML content, preceded by \n.
    let mut start = 0;
    while let Some(rel) = s[start..].find("\n---") {
        let abs = start + rel;
        let after = &s[abs + 4..];
        if after.is_empty() {
            return Some((abs, abs + 4));
        }
        if after.starts_with('\n') {
            return Some((abs, abs + 5));
        }
        start = abs + 1;
    }
    None
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn str_val(v: &Value, key: &str) -> String {
        v[key].as_str().unwrap_or("").to_owned()
    }

    #[test]
    fn basic_frontmatter_and_body() {
        let input = "---\ntitle: Hello\ndate: 2024-01-01\n---\nBody text here.";
        let (meta, body) = parse(input).unwrap();
        assert_eq!(str_val(&meta, "title"), "Hello");
        assert_eq!(str_val(&meta, "date"), "2024-01-01");
        assert_eq!(body, "Body text here.");
    }

    #[test]
    fn frontmatter_only_no_body() {
        let input = "---\ntitle: No body\n---\n";
        let (meta, body) = parse(input).unwrap();
        assert_eq!(str_val(&meta, "title"), "No body");
        assert_eq!(body, "");
    }

    #[test]
    fn no_frontmatter_returns_whole_input_as_body() {
        let input = "Just a plain body with no frontmatter.";
        let (meta, body) = parse(input).unwrap();
        assert!(meta.as_mapping().unwrap().is_empty());
        assert_eq!(body, input);
    }

    #[test]
    fn empty_frontmatter_block() {
        let input = "---\n---\nSome body.";
        let (meta, body) = parse(input).unwrap();
        // Empty YAML parses as Null, not a Mapping — treat it as absent.
        assert!(meta.is_null() || meta.as_mapping().map_or(true, |m| m.is_empty()));
        assert_eq!(body, "Some body.");
    }

    #[test]
    fn boolean_and_array_fields() {
        let input = "---\ndraft: true\ntags:\n  - rust\n  - web\n---\nContent.";
        let (meta, body) = parse(input).unwrap();
        assert_eq!(meta["draft"].as_bool(), Some(true));
        let tags: Vec<&str> = meta["tags"]
            .as_sequence()
            .unwrap()
            .iter()
            .map(|v| v.as_str().unwrap())
            .collect();
        assert_eq!(tags, ["rust", "web"]);
        assert_eq!(body, "Content.");
    }

    #[test]
    fn closing_delimiter_requires_own_line() {
        // --- embedded inside a YAML value should not close the frontmatter.
        let input = "---\ntitle: Test\nsubtitle: intro---here\n---\nBody.";
        let (meta, body) = parse(input).unwrap();
        assert_eq!(str_val(&meta, "title"), "Test");
        assert_eq!(str_val(&meta, "subtitle"), "intro---here");
        assert_eq!(body, "Body.");
    }

    #[test]
    fn no_trailing_newline_after_close() {
        let input = "---\ntitle: Compact\n---";
        let (meta, body) = parse(input).unwrap();
        assert_eq!(str_val(&meta, "title"), "Compact");
        assert_eq!(body, "");
    }
}
