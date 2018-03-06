#[macro_use]
extern crate difference;

extern crate cobalt;
extern crate error_chain;
extern crate tempdir;
extern crate walkdir;

use std::error::Error;
use std::fs::{self, File};
use std::io::Read;
use std::path::{Path, PathBuf};

use error_chain::ChainedError;
use tempdir::TempDir;
use walkdir::WalkDir;

macro_rules! assert_contains {
    ($haystack: expr, $needle: expr) => {
        let text = $haystack.to_owned();
        println!("text='''{}'''", text);
        println!("needle='''{}'''", $needle);
        assert!(text.contains($needle))
    }
}

fn assert_dirs_eq(expected: &Path, actual: &Path) {
    // Ensure everything was created.
    let walker = WalkDir::new(&actual)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file());
    for entry in walker {
        let relative = entry
            .path()
            .strip_prefix(&actual)
            .expect("Comparison error");

        let mut original = String::new();
        File::open(entry.path())
            .expect("Comparison error")
            .read_to_string(&mut original)
            .expect("Could not read to string");

        let dest_file = Path::new(expected).join(&relative);
        assert!(dest_file.exists(), "{:?} doesn't exist", dest_file);
        let mut created = String::new();
        File::open(dest_file.as_path())
            .expect("Comparison error")
            .read_to_string(&mut created)
            .expect("Could not read to string");

        assert_diff!(&original, &created, " ", 0);
    }

    // Ensure no unnecessary files were created
    let walker = WalkDir::new(expected)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file());
    for entry in walker {
        let extra_file = entry
            .path()
            .strip_prefix(expected)
            .expect("Comparison error");
        let src_file = Path::new(actual).join(&extra_file);

        File::open(&src_file).expect(&format!(
            "File {:?} does not exist in reference ({:?}).",
            entry.path(),
            src_file
        ));
    }
}

fn run_test(name: &str) -> Result<(), cobalt::Error> {
    let target = format!("tests/target/{}/", name);
    let target: PathBuf = target.into();
    let mut config = cobalt::ConfigBuilder::from_cwd(format!("tests/fixtures/{}", name))?;
    let destdir = TempDir::new(name).expect("Tempdir not created");

    config.source = "./".to_owned();
    config.abs_dest = Some(destdir.path().to_owned());

    let config = config.build()?;
    let destination = config.destination.clone();

    // try to create the target directory, ignore errors
    fs::create_dir_all(&destination).is_ok();

    let result = cobalt::build(config);

    if result.is_ok() {
        assert_dirs_eq(&destination, &target);
    }

    // clean up
    destdir.close()?;

    result
}

#[test]
pub fn copy_files() {
    run_test("copy_files").expect("Build error");
}

#[test]
pub fn custom_paths() {
    run_test("custom_paths").expect("Build error");
}

#[test]
pub fn custom_posts_folder() {
    run_test("custom_posts_folder").expect("Build error");
}

#[test]
pub fn custom_post_path() {
    run_test("custom_post_path").expect("Build error");
}

#[test]
pub fn dotfiles() {
    run_test("dotfiles").expect("Build error");
}

#[test]
pub fn drafts() {
    run_test("drafts").expect("Build error");
}

#[test]
pub fn drafts_not_shown_by_default() {
    run_test("drafts_not_shown_by_default").expect("Build error");
}

#[test]
pub fn example() {
    run_test("example").expect("Build error");
}

#[test]
pub fn hidden_posts_folder() {
    run_test("hidden_posts_folder").expect("Build error");
}

#[test]
pub fn custom_template_extensions() {
    run_test("custom_template_extensions").expect("Build error");
}

#[cfg(feature = "syntax-highlight")]
#[test]
pub fn syntax_highlight() {
    // Syntect isn't thread safe, for now run everything in the same test.
    run_test("syntax_highlight").expect("Build error");

    run_test("syntax_highlight_theme").expect("Build error");
}

#[test]
pub fn incomplete_rss() {
    let err = run_test("incomplete_rss");
    assert!(err.is_err());

    let err = err.unwrap_err();
    assert_eq!(
        format!("{}", err),
        "name, description and link need to be defined in the config file to generate RSS"
    );
    assert_eq!(err.description(), "missing fields in config file");
}

#[test]
pub fn liquid_error() {
    let err = run_test("liquid_error");
    assert!(err.is_err());
    assert_contains!(
        format!("{}", err.unwrap_err().display_chain()),
        "Invalid identifier"
    );
}

#[test]
pub fn liquid_raw() {
    run_test("liquid_escaped").expect("Build error");
}

#[test]
pub fn no_extends_error() {
    let err = run_test("no_extends_error");
    assert!(err.is_err());
    assert_contains!(
        format!("{}", err.unwrap_err().display_chain()),
        "Layout default_nonexistent.liquid does not exist (referenced in \
         \"index.html\")"
    );
}

#[test]
pub fn sort_posts() {
    run_test("sort_posts").expect("Build error");
}

#[test]
pub fn post_order() {
    run_test("post_order").expect("Build error");
}

#[test]
pub fn previous_next() {
    run_test("previous_next").expect("Build error");
}

#[test]
pub fn rss() {
    run_test("rss").expect("Build error");
}

#[test]
pub fn jsonfeed() {
    run_test("jsonfeed").expect("Build error");
}

#[test]
pub fn ignore_files() {
    run_test("ignore_files").unwrap();
}

#[test]
pub fn yaml_error() {
    let err = run_test("yaml_error");
    assert!(err.is_err());
    let error_message = format!("{}", err.unwrap_err().display_chain());
    assert_contains!(error_message, "unexpected character");
}

#[test]
pub fn excerpts() {
    run_test("excerpts").unwrap();
}

#[test]
pub fn excerpts_crlf() {
    run_test("excerpts_CRLF").unwrap();
}

#[test]
pub fn posts_in_subfolder() {
    run_test("posts_in_subfolder").unwrap();
}

#[test]
pub fn empty_frontmatter() {
    run_test("empty_frontmatter").expect("Build error");
}

#[test]
pub fn querystrings() {
    run_test("querystrings").expect("Build error");
}

#[test]
pub fn markdown_table() {
    run_test("markdown_table").expect("Build error");
}

#[cfg(feature = "sass")]
#[test]
pub fn sass() {
    run_test("sass").expect("Build error");
}

#[cfg(feature = "sass")]
#[test]
pub fn sass_custom_config() {
    run_test("sass_custom_config").expect("Build error");
}

#[test]
pub fn data_files() {
    run_test("data_files").expect("Build error");
}

#[test]
pub fn published_date() {
    run_test("published_date").expect("Build error");
}
