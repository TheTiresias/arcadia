use std::fs;

use crate::helpers;

#[test]
fn story_toc_rendered() {
    let (_tmp, dist) = helpers::build_fixture("fiction_story");
    assert!(dist.join("fiction/my-story/index.html").exists());
}

#[test]
fn chapter_pages_rendered() {
    let (_tmp, dist) = helpers::build_fixture("fiction_story");
    assert!(dist.join("fiction/my-story/ch1.html").exists());
    assert!(dist.join("fiction/my-story/ch2.html").exists());
    assert!(dist.join("fiction/my-story/ch3.html").exists());
}

#[test]
fn chapter_has_next_link() {
    let (_tmp, dist) = helpers::build_fixture("fiction_story");
    let html = fs::read_to_string(dist.join("fiction/my-story/ch1.html")).unwrap();
    assert!(html.contains(r#"href="ch2.html""#));
}

#[test]
fn chapter_has_prev_link() {
    let (_tmp, dist) = helpers::build_fixture("fiction_story");
    let html = fs::read_to_string(dist.join("fiction/my-story/ch2.html")).unwrap();
    assert!(html.contains(r#"href="ch1.html""#));
    assert!(html.contains(r#"href="ch3.html""#));
}

#[test]
fn first_chapter_no_prev() {
    let (_tmp, dist) = helpers::build_fixture("fiction_story");
    let html = fs::read_to_string(dist.join("fiction/my-story/ch1.html")).unwrap();
    // prev slot is an inactive span, not a link
    assert!(html.contains(r#"<span class="inactive">←</span>"#));
}

#[test]
fn last_chapter_no_next() {
    let (_tmp, dist) = helpers::build_fixture("fiction_story");
    let html = fs::read_to_string(dist.join("fiction/my-story/ch3.html")).unwrap();
    // next slot is an inactive span, not a link
    assert!(html.contains(r#"<span class="inactive">→</span>"#));
}

#[test]
fn fiction_index_lists_story() {
    let (_tmp, dist) = helpers::build_fixture("fiction_story");
    let html = fs::read_to_string(dist.join("fiction.html")).unwrap();
    assert!(html.contains("My Story"));
    assert!(html.contains("fiction/my-story/index.html"));
}
