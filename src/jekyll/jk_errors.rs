use std::io;
use liquid;
use ignore;
use serde_yaml;
use std::convert::From;

error_chain! {

    links {
       Another(super::error::Error, super::error::ErrorKind);
    }

    foreign_links {
        Io(io::Error);
        Liquid(liquid::Error);
        SerdeYaml(serde_yaml::Error);
        Ignore(ignore::Error);
    }

    errors {
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
