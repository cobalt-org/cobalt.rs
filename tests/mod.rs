extern crate difference;
extern crate cobalt;
extern crate walkdir;

use std::path::Path;
use std::fs::{self, File};
use std::io::Read;
use walkdir::WalkDir;

fn run_test(name: &str) {
    let source = format!("tests/fixtures/{}/", name);
    let target = format!("tests/target/{}/", name);
    let dest = format!("tests/tmp/{}/", name);

    match cobalt::build(&Path::new(&source), &Path::new(&dest), "_layouts", "_posts") {
        Ok(_) => println!("Build successful"),
        Err(e) => panic!("Error: {}", e),
    }

    let walker = WalkDir::new(&target).into_iter();

    for entry in walker.filter_map(|e| e.ok()).filter(|e| e.file_type().is_file()) {
        let relative = entry.path().to_str().unwrap().split(&target).last().unwrap();

        let mut original = String::new();
        File::open(entry.path()).unwrap().read_to_string(&mut original).unwrap();

        println!("{:?}", &Path::new(&dest).join(&relative));
        let mut created = String::new();
        File::open(&Path::new(&dest).join(&relative)).unwrap().read_to_string(&mut created).unwrap();

        difference::assert_diff(&original, &created, " ", 0);
    }

    fs::remove_dir_all(dest).unwrap();
}

#[test]
pub fn example() {
    run_test("example");
}

