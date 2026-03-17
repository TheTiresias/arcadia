use std::fs;

use crate::helpers;

#[test]
fn sidenote_image_gets_lightbox_trigger() {
    let (_tmp, dist) = helpers::build_fixture("sidenote_image");
    let html = fs::read_to_string(dist.join("posts/with-image.html")).unwrap();
    assert!(html.contains("lightbox-trigger"));
}

#[test]
fn sidenote_image_gets_lightbox_overlay() {
    let (_tmp, dist) = helpers::build_fixture("sidenote_image");
    let html = fs::read_to_string(dist.join("posts/with-image.html")).unwrap();
    assert!(html.contains(r#"class="lightbox""#));
    assert!(html.contains("lb-img-1"));
}

#[test]
fn marginnote_image_gets_lightbox_trigger() {
    let (_tmp, dist) = helpers::build_fixture("sidenote_image");
    let html = fs::read_to_string(dist.join("posts/with-image.html")).unwrap();
    assert!(html.contains("lb-img-2"));
}

#[test]
fn lightbox_close_link_present() {
    let (_tmp, dist) = helpers::build_fixture("sidenote_image");
    let html = fs::read_to_string(dist.join("posts/with-image.html")).unwrap();
    assert!(html.contains("lightbox-close"));
}

#[test]
fn plain_sidenote_without_image_unaffected() {
    let (_tmp, dist) = helpers::build_fixture("simple_post");
    let html = fs::read_to_string(dist.join("posts/hello.html")).unwrap();
    assert!(!html.contains("lightbox-trigger"));
    assert!(!html.contains("lb-img-"));
}
