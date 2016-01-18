#![deny(warnings)]

extern crate cobalt;
extern crate getopts;
extern crate yaml_rust;

use getopts::{ Matches, Options };
use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::path::{ Path, PathBuf };
use yaml_rust::Yaml;
use yaml_rust::YamlLoader;
use cobalt::error::Result;

fn print_version() {
    println!("0.1.2");
}

fn main() {
    let args: Vec<String> = env::args().collect();

    let mut opts = Options::new();
    opts.optopt("s",
                "source",
                "Build from example/folder",
                "[example/folder]");
    opts.optopt("d",
                "destination",
                "Build into example/folder/build",
                "[example/folder]");
    opts.optopt("", "layouts", "Folder to get layouts from", "[_layouts]");
    opts.optopt("", "posts", "Folder to get posts from", "[_posts]");
    opts.optflag("h", "help", "Print this help menu");
    opts.optflag("v", "version", "Display version");

    let matches = match opts.parse(&args[1..]) {
        Ok(m) => {
            m
        }
        Err(f) => {
            panic!(f.to_string())
        }
    };

    if matches.opt_present("h") {
        println!("{}", opts.usage("\n\tcobalt build"));
        return;
    }

    if matches.opt_present("v") {
        print_version();
        return;
    }

    // Fetch config information if available
    let config_contents_result = get_config_contents("./.cobalt.yml");
    let yaml = if let Ok(config_contents) = config_contents_result {
        // TODO: call yaml.to_hash
        match YamlLoader::load_from_str(&config_contents) {
            Ok(y) => y[0].clone(),
            Err(e) => {
                // Trouble parsing yaml file
                panic!(e.to_string())
            }
        }
    } else {
        // No config file or error reading it.
        Yaml::from_str("")
    };


    // join("") makes sure path has a trailing slash
    let source = PathBuf::from(&get_setting("s", "source", "./", &matches, &yaml)).join("");
    let dest = PathBuf::from(&get_setting("d", "dest", "./", &matches, &yaml)).join("");
    let layouts = get_setting("layouts", "layouts", "_layouts", &matches, &yaml);
    let posts = get_setting("posts", "posts", "_posts", &matches, &yaml);

    let command = if !matches.free.is_empty() {
        matches.free[0].clone()
    } else {
        println!("{}", opts.usage("\n\tcobalt build"));
        return;
    };

    match command.as_ref() {
        "build" => {
            println!("building from {} into {}", source.display(), dest.display());
            match cobalt::build(&source, &dest, &layouts, &posts) {
                Ok(_) => println!("Build successful"),
                Err(e) => panic!("Error: {}", e),
            }
        }

        _ => {
            println!("{}", opts.usage("\n\tcobalt build"));
            return;
        }
    }
}

fn get_setting(arg_str: &str, config_str: &str, default: &str, matches: &Matches, yaml: &Yaml) -> String {
    if let Some(arg_val) = matches.opt_str(arg_str) {
        arg_val
    } else if let Some(config_val) = yaml[config_str].as_str() {
        config_val.to_string()
    } else {
        default.to_string()
    }
}

fn get_config_contents<P: AsRef<Path>>(config_file: P) -> Result<String> {
    let mut buffer = String::new();
    let mut f = try!(File::open(config_file));
    try!(f.read_to_string(&mut buffer));
    Ok(buffer)
}

// Private method tests

#[test]
fn get_config_contents_ok() {
    let result = get_config_contents("tests/fixtures/config_example/.cobalt.yml");
    assert!(result.is_ok());
    assert!(result.unwrap().len() != 0);
}

#[test]
fn get_config_contents_err() {
    let result = get_config_contents("tests/fixtures/config_example/config_does_not_exist.yml");
    assert!(result.is_err());
}

