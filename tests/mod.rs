extern crate difference;
extern crate cobalt;
extern crate tempdir;
extern crate walkdir;
#[macro_use]
extern crate lazy_static;

use std::path::{Path, PathBuf};
use std::fs::{self, File};
use std::io::Read;
use std::env;
use tempdir::TempDir;
use walkdir::WalkDir;
use std::error::Error;
use cobalt::Config;

lazy_static! {
    // Create a static variable containing the current working directory(CWD).
    // This allows us to change the CWD without forgetting where we were at the begining.
    static ref WORKING_DIRECTORY:PathBuf = env::current_dir().unwrap();
}

fn run_test(name: &str) -> Result<(), cobalt::Error> {
    // Reset working directory in the event the previous test did not.
    try!(env::set_current_dir(WORKING_DIRECTORY.clone()));
    let target = format!("tests/target/{}/", name);
    let mut config = Config::from_file(format!("tests/fixtures/{}/.cobalt.yml", name))
        .unwrap_or(Default::default());
    let destdir = TempDir::new(name).expect("Tempdir not created");
    let srcdir = Path::new("tests/fixtures/").join(name);

    // We should change the working directory if the config defines a non-default source.
    let build_changing_cwd = config.source != "./";
    if !build_changing_cwd {
        config.source = srcdir.to_str()
            .expect("Can't convert source dir to string")
            .to_owned();
    }
    config.dest = destdir.path()
        .to_str()
        .expect("Can't convert destdir to str")
        .to_owned();

    // try to create the target directory, ignore errors
    fs::create_dir_all(&config.dest).is_ok();

    // If we are using a custom `config.source`, then run the build from the directory.
    // and after building reset the working directory.
    if build_changing_cwd {
        let chdir = WORKING_DIRECTORY.join(srcdir);
        try!(env::set_current_dir(chdir));
    }
    let result = cobalt::build(&config);
    if build_changing_cwd {
        try!(env::set_current_dir(WORKING_DIRECTORY.clone()));
    }

    if result.is_ok() {
        let walker = WalkDir::new(&target)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file());

        // walk through fixture and created tmp directory and compare files
        for entry in walker {
            let relative = entry.path()
                .strip_prefix(&target)
                .expect("Comparison error");

            let mut original = String::new();
            let original_path = entry.path();
            File::open(original_path)
                .expect(&format!("Cannot open original path. Comparison error {:?}",
                                 original_path))
                .read_to_string(&mut original)
                .expect("Could not read to string");

            let mut created = String::new();
            let created_path = Path::new(&config.dest).join(&relative);
            File::open(&original_path)
                .expect(&format!("Cannot open created path. Comparison error {:?}",
                                 created_path))
                .read_to_string(&mut created)
                .expect("Could not read to string");

            difference::assert_diff(&original, &created, " ", 0);
        }

        // ensure no unnecessary files were created
        let walker = WalkDir::new(&config.dest)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file());

        for entry in walker {
            let relative = entry.path()
                .strip_prefix(&config.dest)
                .expect("Comparison error");
            let relative = Path::new(&target).join(&relative);

            File::open(&relative).expect(&format!("File {:?} does not exist in reference ({:?}).",
                                                  entry.path(),
                                                  relative));
        }
    }

    // clean up
    try!(destdir.close());

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
    run_test("syntax_highlight").expect("Build error");
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
    assert_eq!(err.unwrap_err().description(),
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
    assert!(err.unwrap_err()
        .description()
        .contains("Layout default_nonexistent.liquid can not be read (defined in \
                   tests/fixtures/no_extends_error/index.liquid)"));
}


/// Tests for a fix for [Issues#183][1]
///  - If the `source` variable is `.`, then additional files are not moved correctly.
///
/// [1]: https://github.com/cobalt-org/cobalt.rs/issues/183
#[test]
pub fn source_set_to_dot() {
    run_test("source_set_to_dot").unwrap();
}

#[test]
pub fn sort_posts() {
    run_test("sort_posts").expect("Build error");
}

#[test]
pub fn rss() {
    run_test("rss").expect("Build error");
}

#[test]
pub fn ignore_files() {
    run_test("ignore_files").unwrap();
}

#[test]
pub fn yaml_error() {
    let err = run_test("yaml_error");
    assert!(err.is_err());
    assert_eq!(err.unwrap_err().description(), "unexpected character: `@'");
}

#[test]
pub fn excerpts() {
    run_test("excerpts").unwrap();
}

#[test]
pub fn posts_in_subfolder() {
    run_test("posts_in_subfolder").unwrap();
}
