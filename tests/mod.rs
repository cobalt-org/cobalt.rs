extern crate difference;
extern crate cobalt;
extern crate walkdir;

use std::path::Path;
use std::fs::{self, File};
use std::io::Read;
use walkdir::WalkDir;
use std::error::Error;
use cobalt::Config;

fn run_test(name: &str) -> Result<(), cobalt::Error> {
    let target = format!("tests/target/{}/", name);
    let mut config = Config::from_file(format!("tests/fixtures/{}/.cobalt.yml", name)).unwrap_or(Default::default());

    config.source = format!("tests/fixtures/{}/", name);
    config.dest = format!("tests/tmp/{}/", name);

    // try to create the target directory, ignore errors
    fs::create_dir_all(&config.dest).is_err();

    let result = cobalt::build(&config);

    if result.is_ok() {
        let walker = WalkDir::new(&target).into_iter();

        // walk through fixture and created tmp directory and compare files
        for entry in walker.filter_map(|e| e.ok()).filter(|e| e.file_type().is_file()) {
            let relative = entry.path().to_str().unwrap().split(&target).last().expect("Comparison error");

            let mut original = String::new();
            File::open(entry.path()).expect("Comparison error").read_to_string(&mut original).unwrap();

            let mut created = String::new();
            File::open(&Path::new(&config.dest).join(&relative))
                .expect("Comparison error")
                .read_to_string(&mut created)
                .unwrap();

            difference::assert_diff(&original, &created, " ", 0);
        }
    }

    // clean up
    fs::remove_dir_all(&config.dest).is_err();

    result
}

#[test]
pub fn dotfiles() {
    run_test("dotfiles").expect("Build error");
}

#[test]
pub fn example() {
    run_test("example").expect("Build error");
}

#[test]
pub fn custom_template_extensions() {
    run_test("custom_template_extensions").expect("Build error");
}

#[test]
pub fn incomplete_rss() {
    let err = run_test("incomplete_rss");
    assert!(err.is_err());
    assert_eq!(err.unwrap_err().description(), "name, description and link need to be defined in the config file to generate RSS");
}

#[test]
pub fn liquid_error() {
    let err = run_test("liquid_error");
    assert!(err.is_err());
    assert_eq!(err.unwrap_err().description(), "{{{ is not a valid identifier");
}

#[test]
pub fn no_extends_error() {
    let err = run_test("no_extends_error");
    assert!(err.is_err());
    assert_eq!(err.unwrap_err().description(), "No extends property in 2014-08-24-my-first-blogpost");
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
pub fn yaml_error() {
    let err = run_test("yaml_error");
    assert!(err.is_err());
    assert_eq!(err.unwrap_err().description(), "unexpected character: `@'");
}
