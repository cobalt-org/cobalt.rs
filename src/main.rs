extern crate cobalt;
extern crate getopts;

use getopts::Options;
use std::env;
use std::path::PathBuf;

fn print_version() {
    // TODO parse this from Cargo.toml
    println!("0.0.1");
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

    // join("") makes sure path has a trailing slash
    let source = PathBuf::from(&matches.opt_str("s").unwrap_or("./".to_string())).join("");
    let dest = PathBuf::from(&matches.opt_str("d").unwrap_or("./".to_string())).join("");
    let layouts = matches.opt_str("layouts").unwrap_or("_layouts".to_string());
    let posts = matches.opt_str("posts").unwrap_or("_posts".to_string());

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
