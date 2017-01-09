#[macro_use]
extern crate assert_cli;
#[macro_use]
extern crate lazy_static;

use std::env;
use std::str;
use std::path::{Path, PathBuf};

static EMPTY: &'static [&'static str] = &[];
lazy_static! {
    static ref _CWD: PathBuf = env::current_dir().unwrap();
    static ref CWD: &'static Path = _CWD.as_path();
    // TODO test on release
    static ref _BIN: PathBuf = CWD.join("target/debug/cobalt");
    static ref BIN: &'static str = _BIN.to_str().unwrap();
}

macro_rules! assert_contains {
    ($utf8: expr, $vec: expr) => {
        let text = str::from_utf8($utf8).unwrap();
        println!("{}", text);
        assert!(text.contains($vec))
    }
}

macro_rules! assert_contains_not {
    ($utf8: expr, $vec: expr) => {
        let text = str::from_utf8($utf8).unwrap();
        println!("{}", text);
        assert!(!text.contains($vec))
    }
}

#[test]
pub fn invalid_calls() {
    println!("Binary: {:?}", BIN.to_owned());
    let output = assert_cli!(&BIN, EMPTY => Error 1).unwrap();
    assert_contains!(&output.stderr, "requires a subcommand");

    let output = assert_cli!(&BIN, &["--nonexistent-argument"] => Error 1).unwrap();
    assert_contains!(&output.stderr,
                     r"Found argument '--nonexistent-argument' which wasn't expected");
}

#[test]
pub fn log_levels() {
    env::set_current_dir(CWD.join("tests/fixtures/example")).unwrap();

    let output1 = assert_cli!(&BIN, &["build", "--trace"] => Success).unwrap();
    assert_contains!(&output1.stderr, "[trace]");
    assert_contains!(&output1.stderr, "[debug]");
    assert_contains!(&output1.stderr, "[info]");

    let output2 = assert_cli!(&BIN, &["build", "-L", "trace"] => Success).unwrap();
    assert_eq!(output1.stderr, output2.stderr);

    let output = assert_cli!(&BIN, &["build", "-L", "debug"] => Success).unwrap();
    assert_contains_not!(&output.stderr, "[trace]");
    assert_contains!(&output.stderr, "[debug]");
    assert_contains!(&output.stderr, "[info]");

    let output = assert_cli!(&BIN, &["build", "-L", "info"] => Success).unwrap();
    assert_contains_not!(&output.stderr, "[trace]");
    assert_contains_not!(&output.stderr, "[debug]");
    assert_contains!(&output.stderr, "[info]");

    assert_cli!(&BIN, &["build", "--silent"] => Success, "").unwrap();
}

#[test]
pub fn clean() {
    env::set_current_dir(CWD.join("tests/fixtures/example")).unwrap();
    assert_cli!(&BIN, &["build", "-d", "./test_dest"] => Success).unwrap();
    assert_eq!(Path::new("./test_dest/").is_dir(), true);

    let output = assert_cli!(&BIN, &["clean", "-d", "./test_dest"] => Success).unwrap();
    assert_eq!(Path::new("./test_dest").is_dir(), false);
    assert_contains!(&output.stderr, "directory \"./test_dest\" removed");
}

#[cfg(not(windows))]
#[test]
pub fn clean_warning() {
    env::set_current_dir(CWD.join("tests/fixtures/example")).unwrap();
    let output = assert_cli!(&BIN, &["clean"] => Error 1).unwrap();
    assert_contains!(&output.stderr,
                     "Destination directory is same as current directory. Cancelling the \
                      operation");
}
