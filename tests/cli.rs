extern crate assert_cli;
#[macro_use]
extern crate lazy_static;
extern crate tempdir;

use std::env;
use std::str;
use std::path::{Path, PathBuf};

use tempdir::TempDir;

lazy_static! {
    static ref _CWD: PathBuf = env::current_dir().unwrap();
    static ref CWD: &'static Path = _CWD.as_path();
    // TODO test on release
    static ref _BIN: PathBuf = CWD.join("target/debug/cobalt");
    static ref BIN: &'static str = _BIN.to_str().unwrap();
}

#[test]
pub fn invalid_calls() {
    println!("Binary: {:?}", BIN.to_owned());
    assert_cli::Assert::command(&[&BIN])
        .fails()
        .stderr()
        .contains("requires a subcommand")
        .unwrap();

    assert_cli::Assert::command(&[&BIN, "--nonexistent-argument"])
        .fails_with(1)
        .stderr()
        .contains(r"Found argument '--nonexistent-argument' which wasn't expected")
        .unwrap();
}

#[test]
pub fn log_levels_trace() {
    let destdir = TempDir::new("trace").expect("Tempdir not created");
    let dest_param = destdir
        .path()
        .to_str()
        .expect("Can't convert destdir to str")
        .to_owned();

    assert_cli::Assert::command(&[&BIN, "build", "-L", "trace", "-d", &dest_param])
        .current_dir(CWD.join("tests/fixtures/example"))
        .stderr()
        .contains("[trace]")
        .stderr()
        .contains("[debug]")
        .stderr()
        .contains("[info]")
        .unwrap();

    destdir.close().unwrap();
}

#[test]
pub fn log_levels_trace_alias() {
    let destdir = TempDir::new("trace").expect("Tempdir not created");
    let dest_param = destdir
        .path()
        .to_str()
        .expect("Can't convert destdir to str")
        .to_owned();

    assert_cli::Assert::command(&[&BIN, "build", "--trace", "-d", &dest_param])
        .current_dir(CWD.join("tests/fixtures/example"))
        .stderr()
        .contains("[trace]")
        .stderr()
        .contains("[debug]")
        .stderr()
        .contains("[info]")
        .unwrap();

    destdir.close().unwrap();
}

#[test]
pub fn log_levels_debug() {
    let destdir = TempDir::new("debug").expect("Tempdir not created");
    let dest_param = destdir
        .path()
        .to_str()
        .expect("Can't convert destdir to str")
        .to_owned();

    assert_cli::Assert::command(&[&BIN, "build", "-L", "debug", "-d", &dest_param])
        .current_dir(CWD.join("tests/fixtures/example"))
        .stderr()
        .doesnt_contain("[trace]")
        .stderr()
        .contains("[debug]")
        .stderr()
        .contains("[info]")
        .unwrap();

    destdir.close().unwrap();
}

#[test]
pub fn log_levels_info() {
    let destdir = TempDir::new("info").expect("Tempdir not created");
    let dest_param = destdir
        .path()
        .to_str()
        .expect("Can't convert destdir to str")
        .to_owned();

    assert_cli::Assert::command(&[&BIN, "build", "-L", "info", "-d", &dest_param])
        .current_dir(CWD.join("tests/fixtures/example"))
        .stderr()
        .doesnt_contain("[trace]")
        .stderr()
        .doesnt_contain("[debug]")
        .stderr()
        .contains("[info]")
        .unwrap();

    destdir.close().unwrap();
}

#[test]
pub fn log_levels_silent() {
    let destdir = TempDir::new("silent").expect("Tempdir not created");
    let dest_param = destdir
        .path()
        .to_str()
        .expect("Can't convert destdir to str")
        .to_owned();

    assert_cli::Assert::command(&[&BIN, "build", "--silent", "-d", &dest_param])
        .current_dir(CWD.join("tests/fixtures/example"))
        .stderr()
        .is("")
        .stdout()
        .is("")
        .unwrap();

    destdir.close().unwrap();
}

#[test]
pub fn clean() {
    let destdir = TempDir::new("clean").expect("Tempdir not created");
    let dest_param = destdir
        .path()
        .to_str()
        .expect("Can't convert destdir to str")
        .to_owned();

    assert_cli::Assert::command(&[&BIN, "build", "--trace", "-d", &dest_param])
        .current_dir(CWD.join("tests/fixtures/example"))
        .unwrap();
    assert_eq!(destdir.path().is_dir(), true);

    assert_cli::Assert::command(&[&BIN, "clean", "--trace", "-d", &dest_param])
        .current_dir(CWD.join("tests/fixtures/example"))
        .unwrap();
    assert_eq!(destdir.path().is_dir(), false);
}

#[cfg(not(windows))]
#[test]
pub fn clean_warning() {
    assert_cli::Assert::command(&[&BIN, "clean"])
        .current_dir(CWD.join("tests/fixtures/example"))
        .fails_with(1)
        .stderr()
        .contains(
            "Attempting to delete current directory, Cancelling the \
              operation",
        )
        .unwrap();
}

#[test]
pub fn init_project_can_build() {
    let initdir = TempDir::new("init").expect("Tempdir not created");

    let destdir = TempDir::new("dest").expect("Tempdir not created");
    let dest_param = destdir
        .path()
        .to_str()
        .expect("Can't convert destdir to str")
        .to_owned();

    assert_cli::Assert::command(&[&BIN, "init", "--trace"])
        .current_dir(initdir.path())
        .unwrap();
    assert_cli::Assert::command(&[&BIN, "build", "--trace", "--drafts", "-d", &dest_param])
        .current_dir(initdir.path())
        .unwrap();

    destdir.close().unwrap();
    initdir.close().unwrap();
}
