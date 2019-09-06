#[macro_use]
extern crate difference;

use std::fs::File;
use std::io::Read;
use std::path::Path;

use assert_fs::prelude::*;
use walkdir::WalkDir;

macro_rules! assert_contains {
    ($haystack:expr, $needle:expr) => {
        let text = $haystack.to_owned();
        println!("text='''{}'''", text);
        println!("needle='''{}'''", $needle);
        assert!(text.contains($needle))
    };
}

fn assert_dirs_eq(expected: &Path, actual: &Path) {
    // Ensure everything was created.
    let walker = WalkDir::new(&expected)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file());
    for entry in walker {
        let relative = entry
            .path()
            .strip_prefix(&expected)
            .expect("Comparison error");

        let mut original = String::new();
        File::open(entry.path())
            .expect("Comparison error")
            .read_to_string(&mut original)
            .expect("Could not read to string");

        let dest_file = Path::new(actual).join(&relative);
        assert!(dest_file.exists(), "{:?} doesn't exist", dest_file);
        let mut created = String::new();
        File::open(dest_file.as_path())
            .expect("Comparison error")
            .read_to_string(&mut created)
            .expect("Could not read to string");

        assert_diff!(&original, &created, " ", 0);
    }

    // Ensure no unnecessary files were created
    let walker = WalkDir::new(actual)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file());
    for entry in walker {
        let extra_file = entry.path().strip_prefix(actual).expect("Comparison error");
        let src_file = Path::new(expected).join(&extra_file);

        File::open(&src_file).expect(&format!(
            "File {:?} does not exist in reference ({:?}).",
            entry.path(),
            src_file
        ));
    }
}

fn run_test(name: &str) -> Result<(), cobalt::Error> {
    let target = assert_fs::TempDir::new().unwrap();
    target
        .copy_from(format!("tests/fixtures/{}", name), &["*"])
        .unwrap();

    let mut config = cobalt::ConfigBuilder::from_cwd(target.path())?;
    config.destination = "./_dest".into();
    let config = config.build()?;
    let destination = config.destination.clone();
    let result = cobalt::build(config);

    if result.is_ok() {
        let expected = format!("tests/target/{}", name);
        let expected_path = Path::new(&expected);
        assert_dirs_eq(expected_path, &destination);
    }
    // Always explicitly close to catch errors, especially on Windows.
    target.close()?;

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
    let err: exitfailure::ExitFailure = err.unwrap_err().into();
    let error_message = format!("{:?}", err);
    assert_contains!(error_message, "base_url");
}

#[test]
pub fn liquid_error() {
    let err = run_test("liquid_error");
    assert!(err.is_err());
}

#[test]
pub fn liquid_raw() {
    run_test("liquid_escaped").expect("Build error");
}

#[test]
pub fn no_extends_error() {
    let err = run_test("no_extends_error");
    assert!(err.is_err());
    let err: exitfailure::ExitFailure = err.unwrap_err().into();
    let error_message = format!("{:?}", err);
    assert_contains!(
        error_message,
        "Layout default_nonexistent.liquid does not exist (referenced in \
         index.html)"
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
    let err: exitfailure::ExitFailure = err.unwrap_err().into();
    let error_message = format!("{:?}", err);
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

#[cfg(feature = "pagination-unstable")]
#[test]
pub fn pagination_all() {
    run_test("pagination_all").expect("Build error");
}

#[cfg(feature = "pagination-unstable")]
#[test]
pub fn pagination_all_reverse_date() {
    run_test("pagination_all_reverse_date").expect("Build error");
}

#[cfg(feature = "pagination-unstable")]
#[test]
pub fn pagination_less_per_page() {
    run_test("pagination_less_per_page").expect("Build error");
}

#[cfg(feature = "pagination-unstable")]
#[test]
pub fn pagination_all_sort_by_title() {
    run_test("pagination_all_sort_by_title").expect("Build error");
}

#[cfg(feature = "pagination-unstable")]
#[test]
pub fn pagination_tags() {
    run_test("pagination_tags").expect("Build error");
}

#[cfg(feature = "pagination-unstable")]
#[test]
pub fn pagination_categories() {
    run_test("pagination_categories").expect("Build error");
}

#[cfg(feature = "pagination-unstable")]
#[test]
pub fn pagination_sort_by_weight() {
    run_test("pagination_sort_by_weight").expect("Build error");
}

#[cfg(feature = "pagination-unstable")]
#[test]
pub fn pagination_dates() {
    run_test("pagination_dates").expect("Build error");
}

#[cfg(feature = "pagination-unstable")]
#[test]
pub fn pagination_compat() {
    run_test("pagination_compat").expect("Build error");
}

#[cfg(feature = "pagination-unstable")]
#[test]
pub fn pagination_compat_deactivated() {
    let err = run_test("pagination_compat_deactivated");
    assert!(err.is_err());
}
