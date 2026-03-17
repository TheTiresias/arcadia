use std::fs;

use crate::helpers;

#[test]
fn tufte_css_is_written_to_dist() {
    let (_tmp, dist) = helpers::build_fixture("simple_post");
    assert!(dist.join("resources/tufte.css").exists());
    let content = fs::read_to_string(dist.join("resources/tufte.css")).unwrap();
    assert!(!content.is_empty());
}

#[test]
fn tufte_css_contains_expected_content() {
    let (_tmp, dist) = helpers::build_fixture("simple_post");
    let content = fs::read_to_string(dist.join("resources/tufte.css")).unwrap();
    assert!(content.contains("et-book"));
}

#[test]
fn ejected_tufte_css_overrides_embedded() {
    let (_tmp, dist) = helpers::build_fixture("ejected_tufte");
    let content = fs::read_to_string(dist.join("resources/tufte.css")).unwrap();
    assert!(content.contains("EJECTED_TUFTE_SENTINEL"));
}
