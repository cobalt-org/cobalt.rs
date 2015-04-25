#![feature(rustc_private)]
#![feature(collections)]

extern crate cobalt;
extern crate getopts;

use getopts::{optopt, optflag, getopts, usage};
use std::env;
use std::path::PathBuf;

fn print_version() {
    // TODO parse this from Cargo.toml
    println!("0.0.1");
}

fn main() {
    let args: Vec<String> = env::args().collect();

    let opts = [
        optopt("s", "source", "Build from example/folder", "[example/folder]"),
        optopt("d", "destination", "Build into example/folder/build", "[example/folder]"),
        optopt("", "layouts", "Folder to get layouts from", "[_layouts]"),
        optopt("", "posts", "Folder to get posts from", "[_posts]"),
        optflag("h", "help", "Print this help menu"),
        optflag("v", "version", "Display version")
    ];

    let matches = match getopts(args.tail(), &opts) {
        Ok(m) => { m }
        Err(f) => { panic!(f.to_string()) }
    };

    if matches.opt_present("h") {
        println!("{}", usage("\n\tcobalt build", &opts));
        return;
    }

    if matches.opt_present("v") {
        print_version();
        return;
    }

    let mut source_buf = PathBuf::new();

    if matches.opt_present("s") {
        source_buf.push(&matches.opt_str("s").unwrap())
    } else {
        source_buf.push("./")
    };

    let source = source_buf.as_path();

    let mut dest_buf = PathBuf::new();

    if matches.opt_present("d") {
        dest_buf.push(&matches.opt_str("d").unwrap())
    } else {
        dest_buf.push("./")
    };

    let dest = dest_buf.as_path();

    let layouts = match matches.opt_str("layouts") {
        Some(x) => x,
        None => "_layouts".to_string(),
    };

    let posts = match matches.opt_str("posts") {
        Some(x) => x,
        None => "_posts".to_string(),
    };

    let command = if !matches.free.is_empty() {
        matches.free[0].clone()
    } else {
        println!("{}", usage("\n\tcobalt build", &opts));
        return;
    };

    match command.as_ref() {
        "build" => {
            println!("building from {} into {}", source.display(), dest.display());
            match cobalt::build(&source, &dest, &layouts, &posts){
                Ok(_) => println!("Build successful"),
                Err(e) => println!("Error: {}", e)
            };
        },

        _ => {
            println!("{}", usage("\n\tcobalt build", &opts));
            return;
        }
    }
}
