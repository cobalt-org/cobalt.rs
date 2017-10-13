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

use cobalt::Config;

macro_rules! assert_contains {
    ($haystack: expr, $needle: expr) => {
        let text = $haystack.to_owned();
        println!("{}", text);
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

        File::open(&src_file).expect(&format!("File {:?} does not exist in reference ({:?}).",
                                              entry.path(),
                                              src_file));
    }
}

fn run_test(name: &str) -> Result<(), cobalt::Error> {
    let target = format!("tests/target/{}/", name);
    let target: PathBuf = target.into();
    let mut config = Config::from_file(format!("tests/fixtures/{}/.cobalt.yml", name))
        .unwrap_or_default();
    let destdir = TempDir::new(name).expect("Tempdir not created");

    config.source = format!("tests/fixtures/{}/", name);
    config.dest = destdir
        .path()
        .to_str()
        .expect("Can't convert destdir to str")
        .to_owned();

    // try to create the target directory, ignore errors
    fs::create_dir_all(&config.dest).is_ok();

    let result = cobalt::build(&config);

    if result.is_ok() {
        assert_dirs_eq(Path::new(config.dest.as_str()), &target);
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
    assert_eq!(format!("{}", err),
               "name, description and link need to be defined in the config file to generate RSS");
    assert_eq!(err.description(), "missing fields in config file");
}

#[test]
pub fn liquid_error() {
    let err = run_test("liquid_error");
    assert!(err.is_err());
    assert_contains!(format!("{}", err.unwrap_err().display_chain()),
                     "{{{ is not a valid identifier");
}

#[test]
pub fn liquid_raw() {
    run_test("liquid_escaped").expect("Build error");
}

#[test]
pub fn no_extends_error() {
    let err = run_test("no_extends_error");
    assert!(err.is_err());
    assert_contains!(format!("{}", err.unwrap_err().display_chain()),
                     "Layout default_nonexistent.liquid can not be read (defined in \
                   \"index.html\")");
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
    assert_eq!(err.unwrap_err().description(), "scan error");
}

#[test]
pub fn excerpts() {
    run_test("excerpts").unwrap();
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

#[cfg(feature = "sass")]
#[test]
pub fn sass() {
    run_test("sass").expect("Build error");
}
