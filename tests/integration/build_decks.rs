use std::fs;

use crate::helpers;

#[test]
fn deck_renders_html() {
    let (_tmp, dist) = helpers::build_fixture("simple_deck");
    assert!(dist.join("decks/my-deck.html").exists());
}

#[test]
fn deck_has_three_slides() {
    let (_tmp, dist) = helpers::build_fixture("simple_deck");
    let html = fs::read_to_string(dist.join("decks/my-deck.html")).unwrap();
    assert_eq!(html.matches(r#"class="slide""#).count(), 3);
}

#[test]
fn decks_index_lists_deck() {
    let (_tmp, dist) = helpers::build_fixture("simple_deck");
    let html = fs::read_to_string(dist.join("decks.html")).unwrap();
    assert!(html.contains("My Deck"));
    assert!(html.contains("decks/my-deck.html"));
}
