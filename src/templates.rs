/// Embedded templates — baked into the binary at compile time.
/// Each constant holds the raw HTML of one template.
pub const INDEX: &str = include_str!("../embed/index.html");
pub const POST: &str = include_str!("../embed/post.html");
pub const FICTION_INDEX: &str = include_str!("../embed/fiction-index.html");
pub const STORY_TOC: &str = include_str!("../embed/story-toc.html");
pub const CHAPTER: &str = include_str!("../embed/chapter.html");
pub const DECKS_INDEX: &str = include_str!("../embed/decks-index.html");
pub const SLIDE_DECK: &str = include_str!("../embed/slide-deck.html");
pub const TAG_PAGE: &str = include_str!("../embed/tag-page.html");
pub const TAGS_INDEX: &str = include_str!("../embed/tags-index.html");

/// Substitute `{{key}}` placeholders in `template` with the corresponding
/// values from `vars`. Keys are matched exactly; unrecognised placeholders
/// are left unchanged so partial renders are safe to inspect.
pub fn render(template: &str, vars: &[(&str, &str)]) -> String {
    let mut out = template.to_owned();
    for (key, value) in vars {
        let placeholder = ["{{", key, "}}"].concat();
        out = out.replace(&placeholder, value);
    }
    out
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_substitution() {
        assert_eq!(render("Hello {{name}}!", &[("name", "World")]), "Hello World!");
    }

    #[test]
    fn multiple_substitutions() {
        assert_eq!(
            render("{{a}} and {{b}}", &[("a", "foo"), ("b", "bar")]),
            "foo and bar"
        );
    }

    #[test]
    fn unknown_placeholder_left_as_is() {
        assert_eq!(
            render("{{known}} {{unknown}}", &[("known", "hi")]),
            "hi {{unknown}}"
        );
    }

    #[test]
    fn placeholder_repeated() {
        assert_eq!(
            render("{{x}} and {{x}} again", &[("x", "yes")]),
            "yes and yes again"
        );
    }

    #[test]
    fn empty_value_replaces_placeholder() {
        assert_eq!(render("before{{x}}after", &[("x", "")]), "beforeafter");
    }

    #[test]
    fn all_templates_load() {
        // Smoke-test that all include_str! paths resolve and are non-empty.
        assert!(!INDEX.is_empty());
        assert!(!POST.is_empty());
        assert!(!FICTION_INDEX.is_empty());
        assert!(!STORY_TOC.is_empty());
        assert!(!CHAPTER.is_empty());
        assert!(!DECKS_INDEX.is_empty());
        assert!(!SLIDE_DECK.is_empty());
        assert!(!TAG_PAGE.is_empty());
        assert!(!TAGS_INDEX.is_empty());
    }
}
