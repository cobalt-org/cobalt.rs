#![deny(warnings)]

extern crate cobalt;
extern crate getopts;
extern crate env_logger;
extern crate notify;

#[macro_use]
extern crate nickel;

#[macro_use]
extern crate log;

use getopts::Options;
use std::env;
use std::fs;
use cobalt::Config;
use log::{LogRecord, LogLevelFilter};
use env_logger::LogBuilder;
use nickel::{Nickel, StaticFilesHandler};

use notify::{RecommendedWatcher, Error, Watcher};
use std::sync::mpsc::channel;
use std::thread;

fn print_version() {
    println!("0.2.0");
}

fn print_usage(opts: Options) {
    println!("{}",
             opts.usage("\n\tcobalt build\n\tcobalt watch\n\tcobalt serve"));
}

fn main() {
    let args: Vec<String> = env::args().collect();

    let mut opts = Options::new();

    opts.optopt("s", "source", "Source folder, Default: ./", "");
    opts.optopt("d", "destination", "Destination folder, Default: ./", "");
    opts.optopt("c",
                "config",
                "Config file to use, Default: .cobalt.yml",
                "");
    opts.optopt("l",
                "layouts",
                "\tLayout templates folder, Default: _layouts/",
                "");
    opts.optopt("p", "posts", "Posts folder, Default: _posts/", "");

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
        print_usage(opts);
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
            build(&config);
        }

        "serve" => {
            build(&config);
            serve(&config.dest);
        }

        "watch" => {
            build(&config);

            let dest = config.dest.clone();
            thread::spawn(move || {
                serve(&dest);
            });

            let (tx, rx) = channel();

            // Automatically select the best implementation for your platform.
            // You can also access each implementation directly e.g. INotifyWatcher.
            let w: Result<RecommendedWatcher, Error> = Watcher::new(tx);

            match w {
                Ok(mut watcher) => {
                    watcher.watch(&config.source).unwrap();
                    info!("watching {:?}", config.source);

                    loop {
                        match rx.recv() {
                            _ => {
                                info!("Rebuilding...");
                                build(&config);
                            }
                        }
                    }
                }
                Err(_) => error!("Error"),
            }
        }

        _ => {
            print_usage(opts);
            return;
        }
    }
}


fn build(config: &Config) {
    info!("Building from {} into {}", config.source, config.dest);
    match cobalt::build(&config) {
        Ok(_) => info!("Build successful"),
        Err(e) => {
            error!("{}", e);
            error!("Build not successful");
            std::process::exit(1);
        }
    };
}

fn serve(dest: &str) {
    info!("Serving {} through static file server", dest);
    let mut server = Nickel::new();

    server.utilize(StaticFilesHandler::new(dest));

    server.listen("127.0.0.1:3000");
}
