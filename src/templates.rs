use std::fs;
use std::path::Path;
use std::sync::OnceLock;

use regex::Regex;

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

/// Render a template against `vars` in two phases:
///
/// 1. **Conditionals** — `{{#if key}}...{{/if}}` blocks are kept (tags
///    stripped) when `key` maps to a non-empty value, and removed entirely
///    otherwise. Nesting is not supported.
/// 2. **Substitution** — `{{key}}` placeholders are replaced with their
///    values. Unrecognised placeholders are left as-is.
pub fn render(template: &str, vars: &[(&str, &str)]) -> String {
    static IF_RE: OnceLock<Regex> = OnceLock::new();
    let re = IF_RE.get_or_init(|| {
        Regex::new(r"(?s)\{\{#if ([A-Za-z0-9_]+)\}\}(.*?)\{\{/if\}\}").unwrap()
    });

    // Phase 1: conditionals.
    let after_ifs = re.replace_all(template, |caps: &regex::Captures| {
        let key = &caps[1];
        let body = &caps[2];
        let truthy = vars.iter().any(|(k, v)| *k == key && !v.is_empty());
        if truthy { body.to_owned() } else { String::new() }
    });

    // Phase 2: substitution.
    let mut out = after_ifs.into_owned();
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
    fn if_block_kept_when_truthy() {
        assert_eq!(
            render("{{#if x}}hello{{/if}}", &[("x", "1")]),
            "hello"
        );
    }

    #[test]
    fn if_block_removed_when_empty() {
        assert_eq!(
            render("before{{#if x}}hello{{/if}}after", &[("x", "")]),
            "beforeafter"
        );
    }

    #[test]
    fn if_block_removed_when_absent() {
        assert_eq!(
            render("before{{#if x}}hello{{/if}}after", &[]),
            "beforeafter"
        );
    }

    #[test]
    fn if_block_with_substitution_inside() {
        assert_eq!(
            render("{{#if show}}{{val}}{{/if}}", &[("show", "1"), ("val", "hi")]),
            "hi"
        );
    }

    #[test]
    fn if_block_multiline() {
        assert_eq!(
            render("a\n{{#if x}}\nb\n{{/if}}\nc", &[("x", "yes")]),
            "a\n\nb\n\nc"
        );
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
