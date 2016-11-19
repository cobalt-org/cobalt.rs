use std::io;
use yaml_rust::scanner;
use walkdir;
use liquid;

error_chain! {

    links {
    }

    foreign_links {
        io::Error, Io;
        liquid::Error, Liquid;
        walkdir::Error, WalkDir;
        scanner::ScanError, Yaml;
    }

    errors {
    }
}
