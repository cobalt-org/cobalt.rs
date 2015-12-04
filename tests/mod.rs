extern crate difference;
extern crate cobalt;
extern crate walkdir;

use std::path::Path;
use std::fs::{self, File};
use std::io::Read;
use walkdir::WalkDir;
use std::error::Error;

fn run_test(name: &str) -> Result<(), cobalt::Error> {
    let source = format!("tests/fixtures/{}/", name);
    let target = format!("tests/target/{}/", name);
    let dest = format!("tests/tmp/{}/", name);

    let result = cobalt::build(&Path::new(&source), &Path::new(&dest), "_layouts", "_posts");

    if result.is_ok() {
        let walker = WalkDir::new(&target).into_iter();

        // walk through fixture and created tmp directory and compare files
        for entry in walker.filter_map(|e| e.ok()).filter(|e| e.file_type().is_file()) {
            let relative = entry.path().to_str().unwrap().split(&target).last().unwrap();

            let mut original = String::new();
            File::open(entry.path()).unwrap().read_to_string(&mut original).unwrap();

            let mut created = String::new();
            File::open(&Path::new(&dest).join(&relative))
                .unwrap()
                .read_to_string(&mut created)
                .unwrap();

            difference::assert_diff(&original, &created, " ", 0);
        }

        // clean up
        fs::remove_dir_all(dest).expect("Cleanup failed");
    }

    result
}

#[test]
pub fn example() {
    assert!(run_test("example").is_ok());
}

#[test]
pub fn dotfiles() {
    assert!(run_test("dotfiles").is_ok());
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
    assert_eq!(err.unwrap_err().description(), "No @extends line creating _posts/2014-08-24-my-first-blogpost.md");
}
