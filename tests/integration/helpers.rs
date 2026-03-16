use std::path::{Path, PathBuf};

use assert_cmd::Command;
use tempfile::TempDir;

pub fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name)
}

pub fn copy_dir_all(src: &Path, dst: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        if ty.is_dir() {
            copy_dir_all(&entry.path(), &dst.join(entry.file_name()))?;
        } else {
            std::fs::copy(entry.path(), dst.join(entry.file_name()))?;
        }
    }
    Ok(())
}

/// Copy fixture into a fresh TempDir, run `arcadia build`, return (dir, dist path).
/// The TempDir must stay in scope for the dist path to remain valid.
pub fn build_fixture(fixture: &str) -> (TempDir, PathBuf) {
    let tmp = tempfile::tempdir().unwrap();
    copy_dir_all(&fixture_path(fixture), tmp.path()).unwrap();
    let dist = tmp.path().join("dist");
    Command::cargo_bin("arcadia")
        .unwrap()
        .arg("build")
        .current_dir(tmp.path())
        .assert()
        .success();
    (tmp, dist)
}

/// Same as build_fixture but passes `--drafts`.
pub fn build_fixture_with_drafts(fixture: &str) -> (TempDir, PathBuf) {
    let tmp = tempfile::tempdir().unwrap();
    copy_dir_all(&fixture_path(fixture), tmp.path()).unwrap();
    let dist = tmp.path().join("dist");
    Command::cargo_bin("arcadia")
        .unwrap()
        .args(["build", "--drafts"])
        .current_dir(tmp.path())
        .assert()
        .success();
    (tmp, dist)
}
