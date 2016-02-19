#![deny(warnings)]

extern crate cobalt;
extern crate getopts;
extern crate env_logger;

#[macro_use]
extern crate log;

use getopts::Options;
use std::env;
use std::fs;
use cobalt::Config;
use log::{LogRecord, LogLevelFilter};
use env_logger::LogBuilder;

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
    opts.optopt("", "config", "Config file to use", "[.cobalt.yml]");
    opts.optopt("", "layouts", "Folder to get layouts from", "[_layouts]");
    opts.optopt("", "posts", "Folder to get posts from", "[_posts]");
    opts.optflag("", "debug", "Log verbose (debug level) information");
    opts.optflag("", "trace", "Log ultra-verbose (trace level) information");
    opts.optflag("", "silent", "Suppress all output");
    opts.optflag("h", "help", "Print this help menu");
    opts.optflag("v", "version", "Display version");

    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(f) => panic!(f.to_string()),
    };

    if matches.opt_present("h") {
        println!("{}", opts.usage("\n\tcobalt build"));
        return;
    }

    if matches.opt_present("version") {
        print_version();
        return;
    }

    let format = |record: &LogRecord| {
        let level = format!("[{}]", record.level()).to_lowercase();
        format!("{:8} {}", level, record.args())
    };

    let mut builder = LogBuilder::new();
    builder.format(format);
    builder.filter(None, LogLevelFilter::Info);

    if matches.opt_present("debug") {
        builder.filter(None, LogLevelFilter::Debug);
    }

    if matches.opt_present("trace") {
        builder.filter(None, LogLevelFilter::Trace);
    }

    if matches.opt_present("silent") {
        builder.filter(None, LogLevelFilter::Off);
    }

    builder.init().unwrap();

    let config_path = match matches.opt_str("config") {
        Some(config) => config,
        None => "./.cobalt.yml".to_owned(),
    };

    // Fetch config information if available
    let mut config: Config = if fs::metadata(&config_path).is_ok() {
        info!("Using config file {}", &config_path);

        match Config::from_file(&config_path) {
            Ok(config) => config,
            Err(e) => {
                error!("Error reading config file:");
                error!("{}", e);
                std::process::exit(1);
            }
        }
    } else {
        Default::default()
    };

    if let Some(source) = matches.opt_str("s") {
        config.source = source;
    };

    if let Some(dest) = matches.opt_str("d") {
        config.dest = dest;
    };

    if let Some(layouts) = matches.opt_str("layouts") {
        config.layouts = layouts;
    };

    if let Some(posts) = matches.opt_str("posts") {
        config.posts = posts;
    };

    let command = if !matches.free.is_empty() {
        matches.free[0].clone()
    } else {
        println!("{}", opts.usage("\n\tcobalt build"));
        return;
    };

    match command.as_ref() {
        "build" => {
            info!("Building from {} into {}", config.source, config.dest);
            match cobalt::build(&config) {
                Ok(_) => info!("Build successful"),
                Err(e) => {
                    error!("{}", e);
                    error!("Build not successful");
                    std::process::exit(1);
                }
            }
        }

        _ => {
            println!("{}", opts.usage("\n\tcobalt build"));
            return;
        }
    }
}
