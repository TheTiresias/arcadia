use std::fs;
use std::path::Path;

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

/// Runtime template set — loaded once per build.
/// Each field is either the project-local override or the embedded constant.
pub struct Templates {
    pub index: String,
    pub post: String,
    pub fiction_index: String,
    pub story_toc: String,
    pub chapter: String,
    pub decks_index: String,
    pub slide_deck: String,
    pub tag_page: String,
    pub tags_index: String,
}

impl Templates {
    pub fn load(project_dir: &Path) -> Self {
        let load = |name: &str, fallback: &str| -> String {
            let path = project_dir.join("embed").join(name);
            fs::read_to_string(&path).unwrap_or_else(|_| fallback.to_owned())
        };
        Templates {
            index: load("index.html", INDEX),
            post: load("post.html", POST),
            fiction_index: load("fiction-index.html", FICTION_INDEX),
            story_toc: load("story-toc.html", STORY_TOC),
            chapter: load("chapter.html", CHAPTER),
            decks_index: load("decks-index.html", DECKS_INDEX),
            slide_deck: load("slide-deck.html", SLIDE_DECK),
            tag_page: load("tag-page.html", TAG_PAGE),
            tags_index: load("tags-index.html", TAGS_INDEX),
        }
    }
}

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
