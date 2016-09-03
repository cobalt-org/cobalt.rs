extern crate difference;
extern crate cobalt;
extern crate walkdir;

use std::path::Path;
use std::fs::{self, File};
use std::io::Read;
use walkdir::WalkDir;
use std::error::Error;
use cobalt::Config;

fn configure(name: &str) -> Result<Config, cobalt::Error> {
    let mut config = Config::from_file(format!("tests/fixtures/{}/.cobalt.yml", name))
                         .unwrap_or(Default::default());

    config.source = format!("tests/fixtures/{}/", name);
    config.dest = format!("tests/tmp/{}/", name);
    Ok(config)
}

fn get_target(name: &str) -> String {
    format!("tests/target/{}/", name)
}

fn setup(name: &str, config: &Config) -> Result<(), cobalt::Error> {
    let target = &get_target(name);
    // try to create the target directory, ignore errors
    fs::create_dir_all(&config.dest).is_ok();

    let result = cobalt::build(&config);

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
            File::open(entry.path())
                .expect("Comparison error")
                .read_to_string(&mut original)
                .expect("Could not read to string");

            let mut created = String::new();
            File::open(&Path::new(&config.dest).join(&relative))
                .expect("Comparison error")
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

            File::open(&relative)
                .expect(&format!("File {:?} does not exist in reference ({:?}).", entry.path(), relative));
        }
    }

    result
}

fn cleanup(config: &Config) {
    fs::remove_dir_all(&config.dest).is_ok();
}

fn run_test(name: &str) -> Result<(), cobalt::Error> {
    let config = try!(configure(name));
    let result = setup(name, &config);
    cleanup(&config);

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

#[test]
pub fn incomplete_rss() {
    let err = run_test("incomplete_rss");
    assert!(err.is_err());
    assert_eq!(err.unwrap_err().description(),
               "name, description and link need to be defined in the config file to generate RSS");
}

#[test]
pub fn liquid_error() {
    let err = run_test("liquid_error");
    assert!(err.is_err());
    assert_eq!(err.unwrap_err().description(),
               "{{{ is not a valid identifier");
}

#[test]
pub fn no_extends_error() {
    let err = run_test("no_extends_error");
    assert!(err.is_err());
    assert!(err
            .unwrap_err()
            .description()
            .contains("Layout default_nonexistent.liquid can not be read (defined in tests/fixtures/no_extends_error/index.liquid)")
            );
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
pub fn subdirectory_posts() {
    let config = configure("subdirectory_posts").expect("configure error");
    setup("subdirectory_posts", &config).expect("build error");
    let index_html = "tests/tmp/subdirectory_posts/index.html";
    let mut index_file = File::open(index_html).expect("cannot open index.html");
    let mut content = String::new();
    index_file.read_to_string(&mut content).expect("cannot read index.html");

    assert!(content.find("sub-post-1.html") != None);
    assert!(content.find("sub-post-2.html") != None);
    assert!(content.find("sub-post-3.html") != None);
    assert!(content.find("sub-post-4.html") != None);

    cleanup(&config);
}
