use std::path::Path;

use assert_cmd::Command;

fn arcadia_new(args: &[&str], dir: &Path) {
    Command::cargo_bin("arcadia")
        .unwrap()
        .arg("new")
        .args(args)
        .current_dir(dir)
        .assert()
        .success();
}

#[test]
fn new_scaffolds_dirs() {
    let tmp = tempfile::tempdir().unwrap();
    arcadia_new(&[], tmp.path());
    assert!(tmp.path().join("example/posts").exists());
    assert!(tmp.path().join("example/fiction").exists());
    assert!(tmp.path().join("example/decks").exists());
    assert!(tmp.path().join("example/resources").exists());
    assert!(tmp.path().join("example/images").exists());
}

#[test]
fn new_creates_config() {
    let tmp = tempfile::tempdir().unwrap();
    arcadia_new(&[], tmp.path());
    let config = tmp.path().join("arcadia.toml");
    assert!(config.exists());
    let content = std::fs::read_to_string(config).unwrap();
    assert!(content.contains("title"));
}

#[test]
fn new_post_creates_file() {
    let tmp = tempfile::tempdir().unwrap();
    arcadia_new(&["post", "my-post"], tmp.path());
    assert!(tmp.path().join("example/posts/my-post.md").exists());
}

#[test]
fn new_deck_creates_file() {
    let tmp = tempfile::tempdir().unwrap();
    arcadia_new(&["deck", "my-deck"], tmp.path());
    assert!(tmp.path().join("example/decks/my-deck.md").exists());
}

#[test]
fn new_fiction_creates_story() {
    let tmp = tempfile::tempdir().unwrap();
    arcadia_new(&["fiction", "my-story"], tmp.path());
    assert!(tmp.path().join("example/fiction/my-story/story.md").exists());
    assert!(tmp.path().join("example/fiction/my-story/chapter-01.md").exists());
}
