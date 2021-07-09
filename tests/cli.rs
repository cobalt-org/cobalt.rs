extern crate assert_cmd;
extern crate assert_fs;
extern crate predicates;

use std::process;

use assert_cmd::prelude::*;
use assert_fs::prelude::*;
use predicates::prelude::*;

#[test]
pub fn invalid_calls() {
    process::Command::cargo_bin("cobalt")
        .unwrap()
        .assert()
        .failure()
        .stderr(predicate::str::contains("requires a subcommand").from_utf8());

    process::Command::cargo_bin("cobalt")
        .unwrap()
        .arg("--nonexistent-argument")
        .assert()
        .failure()
        .stderr(predicate::str::contains("--nonexistent-argument").from_utf8());
}

#[test]
pub fn log_levels_trace() {
    let project_root = assert_fs::TempDir::new().unwrap();
    project_root
        .copy_from("tests/fixtures/example", &["**"])
        .unwrap();

    process::Command::cargo_bin("cobalt")
        .unwrap()
        .args(&["build", "-L", "trace"])
        .current_dir(project_root.path())
        .assert()
        .success()
        .stderr(predicate::str::contains("TRACE").from_utf8())
        .stderr(predicate::str::contains("DEBUG").from_utf8())
        .stderr(predicate::str::contains("INFO").from_utf8());

    project_root.close().unwrap();
}

#[test]
pub fn log_levels_trace_alias() {
    let project_root = assert_fs::TempDir::new().unwrap();
    project_root
        .copy_from("tests/fixtures/example", &["**"])
        .unwrap();

    process::Command::cargo_bin("cobalt")
        .unwrap()
        .args(&["build", "--trace"])
        .current_dir(project_root.path())
        .assert()
        .success()
        .stderr(predicate::str::contains("TRACE").from_utf8())
        .stderr(predicate::str::contains("DEBUG").from_utf8())
        .stderr(predicate::str::contains("INFO").from_utf8());

    project_root.close().unwrap();
}

#[test]
pub fn log_levels_debug() {
    let project_root = assert_fs::TempDir::new().unwrap();
    project_root
        .copy_from("tests/fixtures/example", &["**"])
        .unwrap();

    process::Command::cargo_bin("cobalt")
        .unwrap()
        .args(&["build", "-L", "debug"])
        .current_dir(project_root.path())
        .assert()
        .success()
        .stderr(predicate::str::contains("[trace]").not().from_utf8())
        .stderr(predicate::str::contains("[debug]").from_utf8())
        .stderr(predicate::str::contains("[info]").from_utf8());

    project_root.close().unwrap();
}

#[test]
pub fn log_levels_info() {
    let project_root = assert_fs::TempDir::new().unwrap();
    project_root
        .copy_from("tests/fixtures/example", &["**"])
        .unwrap();

    process::Command::cargo_bin("cobalt")
        .unwrap()
        .args(&["build", "-L", "info"])
        .current_dir(project_root.path())
        .assert()
        .success()
        .stderr(predicate::str::contains("[trace]").not().from_utf8())
        .stderr(predicate::str::contains("[debug]").not().from_utf8())
        .stderr(predicate::str::contains("[info]").from_utf8());

    project_root.close().unwrap();
}

#[test]
pub fn log_levels_silent() {
    let project_root = assert_fs::TempDir::new().unwrap();
    project_root
        .copy_from("tests/fixtures/example", &["**"])
        .unwrap();

    process::Command::cargo_bin("cobalt")
        .unwrap()
        .args(&["build", "--silent"])
        .current_dir(project_root.path())
        .assert()
        .success()
        .stdout(predicate::str::is_empty().from_utf8())
        .stderr(predicate::str::is_empty().from_utf8());

    project_root.close().unwrap();
}

#[test]
pub fn clean() {
    let project_root = assert_fs::TempDir::new().unwrap();
    project_root
        .copy_from("tests/fixtures/example", &["**"])
        .unwrap();
    let dest = project_root.child("_dest");
    dest.assert(predicate::path::missing());

    process::Command::cargo_bin("cobalt")
        .unwrap()
        .args(&["build", "--trace", "-d", "_dest"])
        .current_dir(project_root.path())
        .assert()
        .success();
    dest.assert(predicate::path::exists());

    process::Command::cargo_bin("cobalt")
        .unwrap()
        .args(&["clean", "--trace", "-d", "_dest"])
        .current_dir(project_root.path())
        .assert()
        .success();
    dest.assert(predicate::path::missing());

    project_root.close().unwrap();
}

#[test]
pub fn clean_empty() {
    let project_root = assert_fs::TempDir::new().unwrap();
    project_root
        .copy_from("tests/fixtures/example", &["**"])
        .unwrap();
    let dest = project_root.child("_dest");
    dest.assert(predicate::path::missing());

    process::Command::cargo_bin("cobalt")
        .unwrap()
        .args(&["clean", "--trace", "-d", "_dest"])
        .current_dir(project_root.path())
        .assert()
        .success();
    dest.assert(predicate::path::missing());

    project_root.close().unwrap();
}

#[test]
pub fn init_project_can_build() {
    let project_root = assert_fs::TempDir::new().unwrap();
    let dest = project_root.child("_dest");
    dest.assert(predicate::path::missing());

    process::Command::cargo_bin("cobalt")
        .unwrap()
        .args(&["init", "--trace"])
        .current_dir(project_root.path())
        .assert()
        .success();
    dest.assert(predicate::path::missing());

    process::Command::cargo_bin("cobalt")
        .unwrap()
        .args(&["build", "--trace", "-d", "_dest", "--drafts"])
        .current_dir(project_root.path())
        .assert()
        .success();
    dest.assert(predicate::path::exists());

    project_root.close().unwrap();
}

#[test]
pub fn new_page_can_build() {
    let project_root = assert_fs::TempDir::new().unwrap();
    let dest = project_root.child("_dest");
    dest.assert(predicate::path::missing());

    process::Command::cargo_bin("cobalt")
        .unwrap()
        .args(&["init", "--trace"])
        .current_dir(project_root.path())
        .assert()
        .success();

    process::Command::cargo_bin("cobalt")
        .unwrap()
        .args(&["new", "--trace", "My New Special Page"])
        .current_dir(project_root.path())
        .assert()
        .success();
    project_root
        .child("my-new-special-page.md")
        .assert(predicate::path::exists());

    dest.assert(predicate::path::missing());
    process::Command::cargo_bin("cobalt")
        .unwrap()
        .args(&["build", "--trace", "-d", "_dest", "--drafts"])
        .current_dir(project_root.path())
        .assert()
        .success();
    dest.assert(predicate::path::exists());

    project_root.close().unwrap();
}

#[test]
pub fn new_post_can_build() {
    let project_root = assert_fs::TempDir::new().unwrap();
    let dest = project_root.child("_dest");
    dest.assert(predicate::path::missing());

    process::Command::cargo_bin("cobalt")
        .unwrap()
        .args(&["init", "--trace"])
        .current_dir(project_root.path())
        .assert()
        .success();

    process::Command::cargo_bin("cobalt")
        .unwrap()
        .args(&["new", "--trace", "My New Special Post"])
        .current_dir(project_root.path().join("posts"))
        .assert()
        .success();
    project_root
        .child("posts/my-new-special-post.md")
        .assert(predicate::path::exists());

    dest.assert(predicate::path::missing());
    process::Command::cargo_bin("cobalt")
        .unwrap()
        .args(&["build", "--trace", "-d", "_dest", "--drafts"])
        .current_dir(project_root.path())
        .assert()
        .success();
    dest.assert(predicate::path::exists());

    project_root.close().unwrap();
}

#[test]
pub fn rename_page_can_build() {
    let project_root = assert_fs::TempDir::new().unwrap();
    let dest = project_root.child("_dest");
    dest.assert(predicate::path::missing());

    process::Command::cargo_bin("cobalt")
        .unwrap()
        .args(&["init", "--trace"])
        .current_dir(project_root.path())
        .assert()
        .success();

    process::Command::cargo_bin("cobalt")
        .unwrap()
        .args(&["new", "--trace", "My New Special Page"])
        .current_dir(project_root.path())
        .assert()
        .success();
    project_root
        .child("my-new-special-page.md")
        .assert(predicate::path::exists());

    process::Command::cargo_bin("cobalt")
        .unwrap()
        .args(&[
            "rename",
            "--trace",
            "my-new-special-page.md",
            "New and Improved!",
        ])
        .current_dir(project_root.path())
        .assert()
        .success();
    project_root
        .child("my-special-page.md")
        .assert(predicate::path::missing());
    project_root
        .child("new-and-improved.md")
        .assert(predicate::path::exists());

    dest.assert(predicate::path::missing());
    process::Command::cargo_bin("cobalt")
        .unwrap()
        .args(&["build", "--trace", "-d", "_dest", "--drafts"])
        .current_dir(project_root.path())
        .assert()
        .success();
    dest.assert(predicate::path::exists());

    project_root.close().unwrap();
}

#[test]
pub fn rename_post_can_build() {
    let project_root = assert_fs::TempDir::new().unwrap();
    let dest = project_root.child("_dest");
    dest.assert(predicate::path::missing());

    process::Command::cargo_bin("cobalt")
        .unwrap()
        .args(&["init", "--trace"])
        .current_dir(project_root.path())
        .assert()
        .success();

    process::Command::cargo_bin("cobalt")
        .unwrap()
        .args(&["new", "--trace", "My New Special Post"])
        .current_dir(project_root.path().join("posts"))
        .assert()
        .success();
    project_root
        .child("posts/my-new-special-post.md")
        .assert(predicate::path::exists());

    process::Command::cargo_bin("cobalt")
        .unwrap()
        .args(&[
            "rename",
            "--trace",
            "my-new-special-post.md",
            "New and Improved!",
        ])
        .current_dir(project_root.path().join("posts"))
        .assert()
        .success();
    project_root
        .child("posts/my-new-special-post.md")
        .assert(predicate::path::missing());
    project_root
        .child("posts/new-and-improved.md")
        .assert(predicate::path::exists());

    dest.assert(predicate::path::missing());
    process::Command::cargo_bin("cobalt")
        .unwrap()
        .args(&["build", "--trace", "-d", "_dest", "--drafts"])
        .current_dir(project_root.path())
        .assert()
        .success();
    dest.assert(predicate::path::exists());

    project_root.close().unwrap();
}

#[test]
pub fn publish_post_can_build() {
    let project_root = assert_fs::TempDir::new().unwrap();
    let dest = project_root.child("_dest");
    dest.assert(predicate::path::missing());

    process::Command::cargo_bin("cobalt")
        .unwrap()
        .args(&["init", "--trace"])
        .current_dir(project_root.path())
        .assert()
        .success();
    project_root
        .child("_cobalt.yml")
        .write_str(
            "
site:
  title: cobalt blog
  description: Blog Posts Go Here
  base_url: http://example.com
posts:
  rss: rss.xml
  publish_date_in_filename: false
",
        )
        .unwrap();

    process::Command::cargo_bin("cobalt")
        .unwrap()
        .args(&["new", "--trace", "My New Special Post"])
        .current_dir(project_root.path().join("posts"))
        .assert()
        .success();
    project_root
        .child("posts/my-new-special-post.md")
        .assert(predicate::path::exists());

    process::Command::cargo_bin("cobalt")
        .unwrap()
        .args(&["publish", "--trace", "my-new-special-post.md"])
        .current_dir(project_root.path().join("posts"))
        .assert()
        .success();
    project_root
        .child("posts/my-new-special-post.md")
        .assert(predicate::path::exists());

    dest.assert(predicate::path::missing());
    process::Command::cargo_bin("cobalt")
        .unwrap()
        .args(&["build", "--trace", "-d", "_dest", "--drafts"])
        .current_dir(project_root.path())
        .assert()
        .success();
    dest.assert(predicate::path::exists());

    project_root.close().unwrap();
}

#[test]
pub fn publish_date_in_post() {
    let project_root = assert_fs::TempDir::new().unwrap();
    let dest = project_root.child("_dest");
    dest.assert(predicate::path::missing());

    process::Command::cargo_bin("cobalt")
        .unwrap()
        .args(&["init", "--trace"])
        .current_dir(project_root.path())
        .assert()
        .success();
    project_root
        .child("_cobalt.yml")
        .write_str(
            "
site:
  title: cobalt blog
  description: Blog Posts Go Here
  base_url: http://example.com
posts:
  rss: rss.xml
  publish_date_in_filename: true
",
        )
        .unwrap();

    process::Command::cargo_bin("cobalt")
        .unwrap()
        .args(&["new", "--trace", "My New Special Post"])
        .current_dir(project_root.path().join("posts"))
        .assert()
        .success();
    project_root
        .child("posts/my-new-special-post.md")
        .assert(predicate::path::exists());

    process::Command::cargo_bin("cobalt")
        .unwrap()
        .args(&["publish", "--trace", "my-new-special-post.md"])
        .current_dir(project_root.path().join("posts"))
        .assert()
        .success();
    project_root
        .child("posts/my-new-special-post.md")
        .assert(predicate::path::missing());

    dest.assert(predicate::path::missing());
    process::Command::cargo_bin("cobalt")
        .unwrap()
        .args(&["build", "--trace", "-d", "_dest", "--drafts"])
        .current_dir(project_root.path())
        .assert()
        .success();
    dest.assert(predicate::path::exists());

    project_root.close().unwrap();
}

#[test]
pub fn publish_draft_moves_dir() {
    let project_root = assert_fs::TempDir::new().unwrap();
    let dest = project_root.child("_dest");
    dest.assert(predicate::path::missing());

    process::Command::cargo_bin("cobalt")
        .unwrap()
        .args(&["init", "--trace"])
        .current_dir(project_root.path())
        .assert()
        .success();
    project_root
        .child("_cobalt.yml")
        .write_str(
            "
site:
  title: cobalt blog
  description: Blog Posts Go Here
  base_url: http://example.com
posts:
  drafts_dir: _drafts
  rss: rss.xml
  publish_date_in_filename: false
",
        )
        .unwrap();
    project_root.child("_drafts").create_dir_all().unwrap();

    process::Command::cargo_bin("cobalt")
        .unwrap()
        .args(&["new", "--trace", "My New Special Post"])
        .current_dir(project_root.path().join("_drafts"))
        .assert()
        .success();
    project_root
        .child("_drafts/my-new-special-post.md")
        .assert(predicate::path::exists());

    process::Command::cargo_bin("cobalt")
        .unwrap()
        .args(&["publish", "--trace", "my-new-special-post.md"])
        .current_dir(project_root.path().join("_drafts"))
        .assert()
        .success();
    project_root
        .child("_drafts/my-new-special-post.md")
        .assert(predicate::path::missing());
    project_root
        .child("posts/my-new-special-post.md")
        .assert(predicate::path::exists());

    dest.assert(predicate::path::missing());
    process::Command::cargo_bin("cobalt")
        .unwrap()
        .args(&["build", "--trace", "-d", "_dest", "--drafts"])
        .current_dir(project_root.path())
        .assert()
        .success();
    dest.assert(predicate::path::exists());

    project_root.close().unwrap();
}
