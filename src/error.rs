#![cfg_attr(feature = "cargo-clippy", allow(redundant_closure))]
use std::io;
use yaml_rust::scanner;
use walkdir;
use liquid;

error_chain! {

    links {
    }

    foreign_links {
        Io(io::Error);
        Liquid(liquid::Error);
        WalkDir(walkdir::Error);
        Yaml(scanner::ScanError);
    }

    errors {
        ConfigFileMissingFields {
            description("missing fields in config file")
            display("name, description and link need to be defined in the config file to \
                    generate RSS")
        }
    }
}
