use std::fs;

use crate::helpers;

#[test]
fn tag_index_rendered() {
    let (_tmp, dist) = helpers::build_fixture("tagged_content");
    assert!(dist.join("tags.html").exists());
    let html = fs::read_to_string(dist.join("tags.html")).unwrap();
    assert!(html.contains("#rust"));
}

#[test]
fn tag_page_rendered() {
    let (_tmp, dist) = helpers::build_fixture("tagged_content");
    assert!(dist.join("tags/rust.html").exists());
}

#[test]
fn tag_page_lists_post() {
    let (_tmp, dist) = helpers::build_fixture("tagged_content");
    let html = fs::read_to_string(dist.join("tags/rust.html")).unwrap();
    assert!(html.contains("Rust Post"));
    assert!(html.contains("posts/rust-post.html"));
}

#[test]
fn tag_page_lists_story() {
    let (_tmp, dist) = helpers::build_fixture("tagged_content");
    let html = fs::read_to_string(dist.join("tags/rust.html")).unwrap();
    assert!(html.contains("Tagged Story"));
    assert!(html.contains("fiction/tagged-story/index.html"));
}
