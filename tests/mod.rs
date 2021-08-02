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

        File::open(&src_file).unwrap_or_else(|e| {
            panic!(
                "File {:?} does not exist in reference ({:?}): {}",
                entry.path(),
                src_file,
                e
            )
        });
    }
}

fn run_test(name: &str) -> Result<(), cobalt::Error> {
    let target = assert_fs::TempDir::new().unwrap();
    target
        .copy_from(format!("tests/fixtures/{}", name), &["**"])
        .unwrap();

    let mut config = cobalt_config::Config::from_cwd(target.path())?;
    config.destination = cobalt_config::RelPath::from_unchecked("./_dest");
    let config = cobalt::cobalt_model::Config::from_config(config)?;
    let result = cobalt::build(config);

    // Always explicitly close to catch errors, especially on Windows.
    target.close()?;

    result
}

fn test_with_expected(name: &str) -> Result<(), cobalt::Error> {
    let target = assert_fs::TempDir::new().unwrap();
    target
        .copy_from(format!("tests/fixtures/{}", name), &["**"])
        .unwrap();

    let mut config = cobalt_config::Config::from_cwd(target.path())?;
    config.destination = cobalt_config::RelPath::from_unchecked("./_dest");
    let config = cobalt::cobalt_model::Config::from_config(config)?;
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
    test_with_expected("copy_files").expect("Build error");
}

#[test]
pub fn custom_paths() {
    test_with_expected("custom_paths").expect("Build error");
}

#[test]
pub fn custom_posts_folder() {
    test_with_expected("custom_posts_folder").expect("Build error");
}

#[test]
pub fn custom_post_path() {
    test_with_expected("custom_post_path").expect("Build error");
}

#[test]
pub fn dotfiles() {
    test_with_expected("dotfiles").expect("Build error");
}

#[test]
pub fn drafts() {
    test_with_expected("drafts").expect("Build error");
}

#[test]
pub fn drafts_not_shown_by_default() {
    test_with_expected("drafts_not_shown_by_default").expect("Build error");
}

#[test]
pub fn example() {
    test_with_expected("example").expect("Build error");
}

#[cfg(feature = "html-minifier")]
#[test]
pub fn example_minified() {
    test_with_expected("example_minified").expect("Build error");
}

#[test]
pub fn hidden_posts_folder() {
    test_with_expected("hidden_posts_folder").expect("Build error");
}

#[test]
pub fn custom_template_extensions() {
    test_with_expected("custom_template_extensions").expect("Build error");
}

#[test]
pub fn incomplete_rss() {
    let err = test_with_expected("incomplete_rss");
    assert!(err.is_err());
    let err: exitfailure::ExitFailure = err.unwrap_err().into();
    let error_message = format!("{:?}", err);
    assert_contains!(error_message, "base_url");
}

#[test]
pub fn liquid_error() {
    let err = test_with_expected("liquid_error");
    assert!(err.is_err());
}

#[test]
pub fn liquid_raw() {
    test_with_expected("liquid_escaped").expect("Build error");
}

#[test]
pub fn no_extends_error() {
    let err = test_with_expected("no_extends_error");
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
    test_with_expected("sort_posts").expect("Build error");
}

#[test]
pub fn post_order() {
    test_with_expected("post_order").expect("Build error");
}

#[test]
pub fn previous_next() {
    test_with_expected("previous_next").expect("Build error");
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
    test_with_expected("ignore_files").unwrap();
}

#[test]
pub fn yaml_error() {
    let err = test_with_expected("yaml_error");
    assert!(err.is_err());
    let error_message = format!("{:?}", err);
    assert_contains!(error_message, "unexpected character");
}

#[test]
pub fn excerpts() {
    test_with_expected("excerpts").unwrap();
}

#[test]
pub fn excerpts_crlf() {
    test_with_expected("excerpts_CRLF").unwrap();
}

#[test]
pub fn posts_in_subfolder() {
    test_with_expected("posts_in_subfolder").unwrap();
}

#[test]
pub fn empty_frontmatter() {
    test_with_expected("empty_frontmatter").expect("Build error");
}

#[test]
pub fn querystrings() {
    test_with_expected("querystrings").expect("Build error");
}

#[test]
pub fn markdown_table() {
    test_with_expected("markdown_table").expect("Build error");
}

#[test]
pub fn vimwiki_not_templated() {
    test_with_expected("vimwiki_not_templated").expect("Build error");
}

#[test]
pub fn vimwiki_not_templated_no_syntax_highlighting() {
    test_with_expected("vimwiki_not_templated_no_syntax").expect("Build error");
}

#[cfg(feature = "sass")]
#[test]
pub fn sass() {
    test_with_expected("sass").expect("Build error");
}

#[cfg(feature = "sass")]
#[test]
pub fn sass_custom_config() {
    test_with_expected("sass_custom_config").expect("Build error");
}

#[test]
pub fn data_files() {
    test_with_expected("data_files").expect("Build error");
}

#[test]
pub fn published_date() {
    test_with_expected("published_date").expect("Build error");
}

#[test]
pub fn sitemap() {
    test_with_expected("sitemap").expect("Build error");
}

#[test]
pub fn pagination_all() {
    test_with_expected("pagination_all").expect("Build error");
}

#[test]
pub fn pagination_all_reverse_date() {
    test_with_expected("pagination_all_reverse_date").expect("Build error");
}

#[test]
pub fn pagination_less_per_page() {
    test_with_expected("pagination_less_per_page").expect("Build error");
}

#[test]
pub fn pagination_all_sort_by_title() {
    test_with_expected("pagination_all_sort_by_title").expect("Build error");
}

#[test]
pub fn pagination_tags() {
    test_with_expected("pagination_tags").expect("Build error");
}

#[test]
pub fn pagination_categories() {
    test_with_expected("pagination_categories").expect("Build error");
}

#[test]
pub fn pagination_sort_by_weight() {
    test_with_expected("pagination_sort_by_weight").expect("Build error");
}

#[test]
pub fn pagination_dates() {
    test_with_expected("pagination_dates").expect("Build error");
}
