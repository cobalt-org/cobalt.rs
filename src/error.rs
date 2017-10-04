#![cfg_attr(feature = "cargo-clippy", allow(redundant_closure))]

use std::io;

use walkdir;
use liquid;
use ignore;
use serde_yaml;

error_chain! {

    links {
    }

    foreign_links {
        Io(io::Error);
        Liquid(liquid::Error);
        WalkDir(walkdir::Error);
        SerdeYaml(serde_yaml::Error);
        Ignore(ignore::Error);
    }

    errors {
        ConfigFileMissingFields {
            description("missing fields in config file")
            display("name, description and link need to be defined in the config file to \
                    generate RSS")
        }

        UnsupportedPlatform(functionality: &'static str, platform: &'static str) {
            description("functionality is not implemented for this platform")
            display("{} is not implemented for the {} platform", functionality, platform)
        }

        MissingFrontStart {
            description("Malformed jekyll document, missing frontmatter start")
            display("Malformed jekyll document, missing frontmatter start")
        }

        MissingFrontmatter {
            description("Malformed jekyll document, missing frontmatter")
            display("Malformed jekyll document, missing frontmatter")
        }

        CantOutputInFile {
            description("Destination must be a directory")
            display("Destination must be a directory")
        }

        InternalError {
            description("Something that was not supposed to happen")
            display("Something that was not supposed to happen")
        }
    }
}
