// Deny warnings, except in dev mode
#![deny(warnings)]
#![cfg_attr(feature="dev", warn(warnings))]

// stuff we want clippy to ignore
#![cfg_attr(feature="cargo-clippy", allow(
        cyclomatic_complexity,
        too_many_arguments,
        ))]

extern crate cobalt;
#[macro_use]
extern crate clap;
extern crate env_logger;
extern crate notify;
extern crate glob;
extern crate ghp;

extern crate hyper;

#[macro_use]
extern crate log;

use clap::{Arg, App, SubCommand, AppSettings};
use std::fs;
use cobalt::Config;
use log::{LogRecord, LogLevelFilter};
use env_logger::LogBuilder;
use hyper::server::{Server, Request, Response};
use hyper::uri::RequestUri;
use ghp::import_dir;
use glob::Pattern;
use cobalt::create_new_project;

use notify::{RecommendedWatcher, Error, Watcher};
use std::sync::mpsc::channel;
use std::thread;
use std::path::PathBuf;
use std::io::prelude::*;
use std::io::Result as IoResult;
use std::fs::File;

fn main() {
    let global_matches = App::new("Cobalt")
        .version(crate_version!())
        .author("Benny Klotz <r3qnbenni@gmail.com>, Johann Hofmann")
        .about("A static site generator written in Rust.")
        .setting(AppSettings::SubcommandRequired)
        .setting(AppSettings::GlobalVersion)
        .arg(Arg::with_name("config")
            .short("c")
            .long("config")
            .value_name("FILE")
            .help("Config file to use [default: .cobalt.yml]")
            .global(true)
            .takes_value(true))
        .arg(Arg::with_name("source")
            .short("s")
            .long("source")
            .value_name("DIR")
            .help("Source folder [default: ./]")
            .global(true)
            .takes_value(true))
        .arg(Arg::with_name("destination")
            .short("d")
            .long("destination")
            .value_name("DIR")
            .help("Destination folder [default: ./]")
            .global(true)
            .takes_value(true))
        .arg(Arg::with_name("layouts")
            .short("l")
            .long("layouts")
            .value_name("DIR")
            .help("Layout templates folder [default: ./_layouts]")
            .global(true)
            .takes_value(true))
        .arg(Arg::with_name("posts")
            .short("p")
            .long("posts")
            .value_name("DIR")
            .help("Posts folder [default: ./posts]")
            .global(true)
            .takes_value(true))
        .arg(Arg::with_name("drafts")
            .long("drafts")
            .help("Include drafts.")
            .global(true)
            .takes_value(false))
        .arg(Arg::with_name("log-level")
            .short("L")
            .long("log-level")
            .possible_values(&["error", "warn", "info", "debug", "trace", "off"])
            .help("Log level [default: info]")
            .global(true)
            .takes_value(true))
        .arg(Arg::with_name("trace")
            .long("trace")
            .help("Log ultra-verbose (trace level) information")
            .global(true)
            .takes_value(false))
        .arg(Arg::with_name("silent")
            .long("silent")
            .help("Suppress all output")
            .global(true)
            .takes_value(false))
        .subcommand(SubCommand::with_name("new")
            .about("create a new cobalt project")
            .arg(Arg::with_name("DIRECTORY")
                .help("Suppress all output")
                .default_value("./")
                .index(1)))
        .subcommand(SubCommand::with_name("build")
            .about("build the cobalt project at the source dir")
            .arg(Arg::with_name("import")
                .short("i")
                .long("import")
                .help("Import after build to gh-pages branch")
                .takes_value(false))
            .arg(Arg::with_name("branch")
                .short("b")
                .long("branch")
                .value_name("BRANCH")
                .help("Branch that will be used to import the site to")
                .default_value("gh-pages")
                .takes_value(true))
            .arg(Arg::with_name("message")
                .short("m")
                .long("message")
                .value_name("COMMIT-MESSAGE")
                .help("Commit message that will be used on import")
                .default_value("cobalt site import")
                .takes_value(true)))
        .subcommand(SubCommand::with_name("clean").about("cleans directory set as destination"))
        .subcommand(SubCommand::with_name("serve")
            .about("build and serve the cobalt project at the source dir")
            .arg(Arg::with_name("port")
                .short("P")
                .long("port")
                .value_name("INT")
                .help("Port to serve from")
                .default_value("3000")
                .takes_value(true)))
        .subcommand(SubCommand::with_name("watch")
            .about("build, serve, and watch the project at the source dir")
            .arg(Arg::with_name("port")
                .short("P")
                .long("port")
                .value_name("INT")
                .help("Port to serve from")
                .default_value("3000")
                .takes_value(true)))
        .subcommand(SubCommand::with_name("import")
            .about("moves the contents of the dest folder to the gh-pages branch")
            .arg(Arg::with_name("branch")
                .short("b")
                .long("branch")
                .value_name("BRANCH")
                .help("Branch that will be used to import the site to")
                .default_value("gh-pages")
                .takes_value(true))
            .arg(Arg::with_name("message")
                .short("m")
                .long("message")
                .value_name("COMMIT-MESSAGE")
                .help("Commit message that will be used on import")
                .default_value("cobalt site import")
                .takes_value(true)))
        .get_matches();

    let (command, matches) = match global_matches.subcommand() {
        (command, Some(matches)) => (command, matches),
        (_, None) => unreachable!(),
    };

    let format = |record: &LogRecord| {
        let level = format!("[{}]", record.level()).to_lowercase();
        format!("{:8} {}", level, record.args())
    };

    let mut builder = LogBuilder::new();
    builder.format(format);

    match matches.value_of("log-level").or(global_matches.value_of("log-level")) {
        Some("error") => builder.filter(None, LogLevelFilter::Error),
        Some("warn") => builder.filter(None, LogLevelFilter::Warn),
        Some("debug") => builder.filter(None, LogLevelFilter::Debug),
        Some("trace") => builder.filter(None, LogLevelFilter::Trace),
        Some("off") => builder.filter(None, LogLevelFilter::Off),
        Some("info") | _ => builder.filter(None, LogLevelFilter::Info),
    };

    if matches.is_present("trace") {
        builder.filter(None, LogLevelFilter::Trace);
    }

    if matches.is_present("silent") {
        builder.filter(None, LogLevelFilter::Off);
    }

    builder.init().unwrap();

    let config_path = matches.value_of("config")
        .or(global_matches.value_of("config"))
        .unwrap_or(".cobalt.yml")
        .to_string();

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
        warn!("No .cobalt.yml file found in current directory, using default config.");
        Default::default()
    };

    config.source = matches.value_of("source")
        .or(global_matches.value_of("source"))
        .map(str::to_string)
        .unwrap_or(config.source);

    config.dest = matches.value_of("destination")
        .or(global_matches.value_of("destination"))
        .map(str::to_string)
        .unwrap_or(config.dest);

    config.layouts = matches.value_of("layouts")
        .or(global_matches.value_of("layouts"))
        .map(str::to_string)
        .unwrap_or(config.layouts);

    config.posts = matches.value_of("posts")
        .or(global_matches.value_of("posts"))
        .map(str::to_string)
        .unwrap_or(config.posts);

    config.include_drafts = matches.is_present("drafts");

    match command {
        "new" => {
            let directory = matches.value_of("DIRECTORY").unwrap();

            match create_new_project(&directory.to_string()) {
                Ok(_) => info!("Created new project at {}", directory),
                Err(e) => {
                    error!("{}", e);
                    error!("Could not create a new cobalt project");
                    std::process::exit(1);
                }
            }
        }

        "build" => {
            build(&config);
            if matches.is_present("import") {
                let branch = matches.value_of("branch").unwrap().to_string();
                let message = matches.value_of("message").unwrap().to_string();
                import(&config, &branch, &message);
            }
        }

        "clean" => {
            let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::new());
            let destdir = std::fs::canonicalize(PathBuf::from(&config.dest))
                .unwrap_or_else(|_| PathBuf::new());
            if cwd == destdir {
                error!("Destination directory is same as current directory. \
                       Cancelling the operation");
                std::process::exit(1);
            }
            match fs::remove_dir_all(&config.dest) {
                Ok(..) => info!("directory \"{}\" removed", &config.dest),
                Err(err) => error!("Error: {}", err),
            }
        }

        "serve" => {
            build(&config);
            let port = matches.value_of("port").unwrap().to_string();
            serve(&config.dest, &port);
        }

        "watch" => {
            build(&config);

            let dest = config.dest.clone();
            let port = matches.value_of("port").unwrap().to_string();
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
                                    // get where process was run from
                                    let cwd = std::env::current_dir()
                                        .unwrap_or_else(|_| PathBuf::new());

                                    // The final goal is to have a relative path. If we already
                                    // have a relative path, we still convert it to an abs path
                                    // first to handle prefix "./" correctly.
                                    let abs_path = if path.is_absolute() {
                                        path.clone()
                                    } else {
                                        cwd.join(&path)
                                    };
                                    let rel_path = abs_path.strip_prefix(&cwd).unwrap_or(&path);

                                    // check whether this path has been marked as ignored in config
                                    let rel_path_matches =
                                        |pattern| Pattern::matches_path(pattern, rel_path);
                                    let path_ignored = &config.ignore.iter().any(rel_path_matches);

                                    if !path_ignored {
                                        build(&config);
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
            let branch = matches.value_of("branch").unwrap().to_string();
            let message = matches.value_of("message").unwrap().to_string();
            import(&config, &branch, &message);
        }

        _ => {
            println!("{}", global_matches.usage());
            return;
        }
    }
}

fn build(config: &Config) {
    info!("Building from {} into {}", config.source, config.dest);
    match cobalt::build(config) {
        Ok(_) => info!("Build successful"),
        Err(e) => {
            error!("{}", e);
            error!("Build not successful");
            std::process::exit(1);
        }
    };
}

fn static_file_handler(dest: &str, req: Request, mut res: Response) -> IoResult<()> {
    // grab the requested path
    let req_path = match req.uri {
        RequestUri::AbsolutePath(p) => p,
        _ => {
            // return a 400 and exit from this request
            *res.status_mut() = hyper::status::StatusCode::BadRequest;
            let body = b"<h1> <center> 400: Bad request </center> </h1>";
            try!(res.send(body));
            return Ok(());
        }
    };

    // find the path of the file in the local system
    // (this gets rid of the '/' in `p`, so the `join()` will not replace the
    // path)
    let path = PathBuf::from(dest).join(&req_path[1..]);

    let serve_path = if path.is_file() {
        // try to point the serve path to `path` if it corresponds to a file
        path
    } else {
        // try to point the serve path into a "index.html" file in the requested
        // path
        path.join("index.html")
    };

    // if the request points to a file and it exists, read and serve it
    if serve_path.exists() {
        let mut file = try!(File::open(serve_path));

        // buffer to store the file
        let mut buffer: Vec<u8> = vec![];

        try!(file.read_to_end(&mut buffer));

        try!(res.send(&buffer));
    } else {
        // return a 404 status
        *res.status_mut() = hyper::status::StatusCode::NotFound;

        // write a simple body for the 404 page
        let body = b"<h1> <center> 404: Page not found </center> </h1>";

        try!(res.send(body));
    }

    Ok(())
}

fn serve(dest: &str, port: &str) {
    info!("Serving {:?} through static file server", dest);

    let ip = format!("127.0.0.1:{}", port);
    info!("Server Listening on {}", &ip);
    info!("Ctrl-c to stop the server");

    // attempts to create a server
    let http_server = match Server::http(&*ip) {
        Ok(server) => server,
        Err(e) => {
            error!("{}", e);
            std::process::exit(1);
        }
    };

    // need a clone because of closure's lifetime
    let dest_clone = dest.to_owned();

    // bind the handle function and start serving
    if let Err(e) = http_server.handle(move |req: Request, res: Response| {
        if let Err(e) = static_file_handler(&dest_clone, req, res) {
            error!("{}", e);
            std::process::exit(1);
        }
    }) {
        error!("{}", e);
        std::process::exit(1);
    };
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
