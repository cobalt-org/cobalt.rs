#![deny(warnings)]

extern crate cobalt;
extern crate getopts;
extern crate env_logger;
extern crate notify;
extern crate glob;
extern crate ghp;

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
use nickel::{Nickel, Options as NickelOptions, StaticFilesHandler};
use ghp::import_dir;
use glob::Pattern;
use cobalt::create_new_project;

use notify::{RecommendedWatcher, Error, Watcher};
use std::sync::mpsc::channel;
use std::thread;
use std::path::PathBuf;

fn print_version() {
    println!("0.2.0");
}

fn print_usage(opts: Options) {
    let usage = concat!("\n\tnew -- create a new cobalt project",
                        "\n\tbuild -- build the cobalt project at the source dir",
                        "\n\tserve -- build and serve the cobalt project at the source dir",
                        "\n\twatch -- build, serve, and watch the project at the source dir",
                        "\n\timport -- moves the contents of the dest folder to the gh-pages branch");
    println!("{}", opts.usage(usage));
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
    opts.optopt("P", "port", "Port to serve from, Default: 3000", "");
    opts.optopt("b", "branch", "Branch that will be used to import the site to, Default: gh-pages", "");
    opts.optopt("m", "message", "Commit message that will be used on import, Default: cobalt site import", "");

    opts.optflag("", "debug", "Log verbose (debug level) information");
    opts.optflag("", "trace", "Log ultra-verbose (trace level) information");
    opts.optflag("", "silent", "Suppress all output");
    opts.optflag("i", "import", "Import after build to gh-pages branch");
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
        print_usage(opts);
        return;
    };

    // Check for port and set port variable to it
    let port = matches.opt_str("port").unwrap_or("3000".to_owned());
    let branch = matches.opt_str("branch").unwrap_or("gh-pages".to_owned());
    let message = matches.opt_str("message").unwrap_or("cobalt site import".to_owned());
    let should_import = matches.opt_present("import");

    match command.as_ref() {
        "build" => {
            build(&config);
            if should_import {
                import(&config, &branch, &message);
            }
        }

        "serve" => {
            build(&config);
            serve(&config.dest, &port);
        }

        "watch" => {
            build(&config);

            let dest = config.dest.clone();
            thread::spawn(move || {
                serve(&dest, &port);
            });

            let (tx, rx) = channel();
            let w: Result<RecommendedWatcher, Error> = Watcher::new(tx);

            match w {
                Ok(mut watcher) => {
                    // TODO: clean up this unwrap
                    watcher.watch(&config.source).unwrap();
                    info!("Watching {:?} for changes", &config.source);

                    loop {
                        match rx.recv() {
                            Ok(val) => {
                                trace!("file changed {:?}", val);
                                if let Some(path) = val.path {
                                    if path.is_absolute() {
                                        // get where process was run from
                                        let cwd = std::env::current_dir().unwrap_or(PathBuf::new());
                                        // strip absolute path
                                        let rel_path = path.strip_prefix(&cwd).unwrap_or(&cwd);

                                        // check if path starts with the build folder.
                                        if !&config.ignore.iter().any(|pattern| Pattern::matches_path(
                                            pattern,
                                            rel_path)) {
                                            build(&config);
                                        }

                                    } else {
                                        // check if path starts with build folder.
                                        // TODO: may want to check if it starts `./`
                                        if !&config.ignore.iter().any(|pattern| Pattern::matches_path(pattern, &path)) {
                                            build(&config);
                                        }
                                    }
                                }
                            }

                            Err(e) => {
                                error!("[Notify Error]: {}", e);
                                std::process::exit(1);
                            }
                        }
                    }
                }
                Err(e) => {
                    error!("[Notify Error]: {}", e);
                    std::process::exit(1);
                }
            }
        }

        "import" => {
            import(&config, &branch, &message);
        }

        "new" => {
            if matches.free.len() == 2 {
                let dest = matches.free[1].clone();

                match create_new_project(&dest) {
                    Ok(_) => info!("Created new project at {}", dest),
                    Err(e) => {
                        error!("{}", e);
                        error!("Could not create a new cobalt project");
                        std::process::exit(1);
                    }
                }
            } else {
                println!("cobalt new DIRECTORY");
                print_usage(opts);
                std::process::exit(1);
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

fn serve(dest: &str, port: &str) {
    info!("Serving {:?} through static file server", dest);
    let mut server = Nickel::new();
    server.options = NickelOptions::default().output_on_listen(false);

    server.utilize(StaticFilesHandler::new(dest));

    let ip = "127.0.0.1:".to_owned() + port;
    info!("Server Listening on {}", &ip);
    info!("Ctrl-c to stop the server");
    server.listen(&*ip);
}

fn import(config: &Config, branch: &str, message: &str) {
    info!("Importing {} to {}", config.dest, branch);

    let meta = match fs::metadata(&config.dest) {
        Ok(data) => data,

        Err(e) => {
            error!("{}", e);
            error!("Import not successful");
            std::process::exit(1);
        }
    };

    if meta.is_dir() {
        match import_dir(&config.dest, branch, message) {
            Ok(_) => info!("Import successful"),
            Err(e) => {
                error!("{}", e);
                error!("Import not successful");
                std::process::exit(1);
            }
        }
    } else {
        error!("Build dir is not a directory: {}", config.dest);
        error!("Import not successful");
        std::process::exit(1);
    }
}
