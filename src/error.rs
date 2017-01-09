#![cfg_attr(feature = "cargo-clippy", allow(redundant_closure))]
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
        ConfigFileMissingFields {
            description("missing fields in config file")
            display("name, description and link need to be defined in the config file to \
                    generate RSS")
        }
    }
}
