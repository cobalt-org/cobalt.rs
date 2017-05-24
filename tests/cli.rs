#[macro_use]
extern crate assert_cli;
#[macro_use]
extern crate lazy_static;
extern crate tempdir;

use std::env;
use std::str;
use std::path::{Path, PathBuf};

use tempdir::TempDir;

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
pub fn log_levels_trace() {
    env::set_current_dir(CWD.join("tests/fixtures/example")).unwrap();

    let destdir = TempDir::new("trace").expect("Tempdir not created");
    let dest_param = destdir
        .path()
        .to_str()
        .expect("Can't convert destdir to str")
        .to_owned();

    let output1 = assert_cli!(&BIN, &["build", "-L", "trace", "-d", &dest_param] => Success)
        .unwrap();
    assert_contains!(&output1.stderr, "[trace]");
    assert_contains!(&output1.stderr, "[debug]");
    assert_contains!(&output1.stderr, "[info]");

    destdir.close().unwrap();
}

#[test]
pub fn log_levels_trace_alias() {
    env::set_current_dir(CWD.join("tests/fixtures/example")).unwrap();

    let destdir = TempDir::new("trace_alias").expect("Tempdir not created");
    let dest_param = destdir
        .path()
        .to_str()
        .expect("Can't convert destdir to str")
        .to_owned();

    let output1 = assert_cli!(&BIN, &["build", "--trace", "-d", &dest_param] => Success).unwrap();
    assert_contains!(&output1.stderr, "[trace]");
    assert_contains!(&output1.stderr, "[debug]");
    assert_contains!(&output1.stderr, "[info]");

    destdir.close().unwrap();
}

#[test]
pub fn log_levels_debug() {
    env::set_current_dir(CWD.join("tests/fixtures/example")).unwrap();

    let destdir = TempDir::new("debug").expect("Tempdir not created");
    let dest_param = destdir
        .path()
        .to_str()
        .expect("Can't convert destdir to str")
        .to_owned();

    let output = assert_cli!(&BIN, &["build", "-L", "debug", "-d", &dest_param] => Success)
        .unwrap();
    assert_contains_not!(&output.stderr, "[trace]");
    assert_contains!(&output.stderr, "[debug]");
    assert_contains!(&output.stderr, "[info]");

    destdir.close().unwrap();
}

#[test]
pub fn log_levels_info() {
    env::set_current_dir(CWD.join("tests/fixtures/example")).unwrap();

    let destdir = TempDir::new("info").expect("Tempdir not created");
    let dest_param = destdir
        .path()
        .to_str()
        .expect("Can't convert destdir to str")
        .to_owned();

    let output = assert_cli!(&BIN, &["build", "-L", "info", "-d", &dest_param] => Success).unwrap();
    assert_contains_not!(&output.stderr, "[trace]");
    assert_contains_not!(&output.stderr, "[debug]");
    assert_contains!(&output.stderr, "[info]");

    destdir.close().unwrap();
}

#[test]
pub fn log_levels_silent() {
    env::set_current_dir(CWD.join("tests/fixtures/example")).unwrap();

    let destdir = TempDir::new("silent").expect("Tempdir not created");
    let dest_param = destdir
        .path()
        .to_str()
        .expect("Can't convert destdir to str")
        .to_owned();

    assert_cli!(&BIN, &["build", "--silent", "-d", &dest_param] => Success, "").unwrap();

    destdir.close().unwrap();
}

#[test]
pub fn clean() {
    env::set_current_dir(CWD.join("tests/fixtures/example")).unwrap();

    let destdir = TempDir::new("clean").expect("Tempdir not created");
    let dest_param = destdir
        .path()
        .to_str()
        .expect("Can't convert destdir to str")
        .to_owned();

    assert_cli!(&BIN, &["build", "-d", &dest_param] => Success).unwrap();
    assert_eq!(destdir.path().is_dir(), true);

    assert_cli!(&BIN, &["clean", "-d", &dest_param] => Success).unwrap();
    assert_eq!(destdir.path().is_dir(), false);
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
