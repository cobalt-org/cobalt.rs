#![cfg_attr(feature = "cargo-clippy", allow(redundant_closure))]
use std::io;
use std::path;
use yaml_rust::scanner;
use walkdir;
use liquid;

error_chain! {

    links {
    }

    foreign_links {
        io::Error, Io;
        path::StripPrefixError, Path;
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
