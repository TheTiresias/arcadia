use std::fs;

use crate::helpers;

#[test]
fn feeds_not_generated_without_base_url() {
    let (_tmp, dist) = helpers::build_fixture("simple_post");
    assert!(!dist.join("feed.xml").exists());
}

#[test]
fn post_feed_generated() {
    let (_tmp, dist) = helpers::build_fixture("with_base_url");
    assert!(dist.join("feed.xml").exists());
}

#[test]
fn post_feed_contains_item() {
    let (_tmp, dist) = helpers::build_fixture("with_base_url");
    let xml = fs::read_to_string(dist.join("feed.xml")).unwrap();
    assert!(xml.contains("Hello World"));
    assert!(xml.contains("https://example.com/posts/hello.html"));
}

#[test]
fn fiction_feed_generated() {
    let (_tmp, dist) = helpers::build_fixture("with_base_url");
    assert!(dist.join("fiction-feed.xml").exists());
}

#[test]
fn decks_feed_generated() {
    let (_tmp, dist) = helpers::build_fixture("with_base_url");
    assert!(dist.join("decks-feed.xml").exists());
}
