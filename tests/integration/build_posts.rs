use std::fs;

use crate::helpers;

#[test]
fn post_renders_html() {
    let (_tmp, dist) = helpers::build_fixture("simple_post");
    assert!(dist.join("posts/hello.html").exists());
}

#[test]
fn post_contains_title() {
    let (_tmp, dist) = helpers::build_fixture("simple_post");
    let html = fs::read_to_string(dist.join("posts/hello.html")).unwrap();
    assert!(html.contains("Hello World"));
}

#[test]
fn post_contains_date() {
    let (_tmp, dist) = helpers::build_fixture("simple_post");
    let html = fs::read_to_string(dist.join("posts/hello.html")).unwrap();
    assert!(html.contains("2024-01-15"));
}

#[test]
fn post_contains_subtitle() {
    let (_tmp, dist) = helpers::build_fixture("simple_post");
    let html = fs::read_to_string(dist.join("posts/hello.html")).unwrap();
    assert!(html.contains("A test post"));
    assert!(html.contains("subtitle"));
}

#[test]
fn draft_excluded_by_default() {
    let (_tmp, dist) = helpers::build_fixture("draft_post");
    assert!(!dist.join("posts/my-draft.html").exists());
    assert!(dist.join("posts/published.html").exists());
}

#[test]
fn draft_included_with_flag() {
    let (_tmp, dist) = helpers::build_fixture_with_drafts("draft_post");
    assert!(dist.join("posts/my-draft.html").exists());
    assert!(dist.join("posts/published.html").exists());
}

#[test]
fn post_index_lists_post() {
    let (_tmp, dist) = helpers::build_fixture("simple_post");
    let html = fs::read_to_string(dist.join("index.html")).unwrap();
    assert!(html.contains("posts/hello.html"));
    assert!(html.contains("Hello World"));
}
