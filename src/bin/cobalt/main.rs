// Deny warnings, except in `dev` mode

// To update this list
// 1. Run `rustc -W help`
// 2. Grab all `default=warn` warnings
// 3. Paste them here, deleting `warnings`, and any with `deprecated` in the name
#![deny(const_err,
        dead_code,
        illegal_floating_point_literal_pattern,
        improper_ctypes,
        non_camel_case_types,
        non_shorthand_field_patterns,
        non_snake_case,
        non_upper_case_globals,
        no_mangle_generic_items,
        overflowing_literals,
        path_statements,
        patterns_in_fns_without_body,
        plugin_as_library,
        private_in_public,
        private_no_mangle_fns,
        private_no_mangle_statics,
        renamed_and_removed_lints,
        stable_features,
        unconditional_recursion,
        unions_with_drop_fields,
        unknown_lints,
        unreachable_code,
        unreachable_patterns,
        unused_allocation,
        unused_assignments,
        unused_attributes,
        unused_comparisons,
        unused_features,
        unused_imports,
        unused_macros,
        unused_must_use,
        unused_mut,
        unused_parens,
        unused_unsafe,
        unused_variables,
        while_true)]
// This list is select `allow` warnings
#![deny(trivial_casts,
       trivial_numeric_casts,
       unused_extern_crates,
       unused_import_braces)]
#![cfg_attr(feature="dev", warn(warnings))]

extern crate cobalt;
extern crate env_logger;
extern crate notify;
extern crate ghp;

extern crate hyper;

#[macro_use]
extern crate error_chain;

#[macro_use]
extern crate clap;

#[macro_use]
extern crate log;

mod build;
mod error;
mod serve;
mod new;

use std::fs;

use clap::{Arg, App, SubCommand, AppSettings};
use cobalt::{Config, Dump};
use cobalt::{list_syntaxes, list_syntax_themes};
use env_logger::LogBuilder;
use log::{LogRecord, LogLevelFilter};

use error::*;

quick_main!(run);

fn run() -> Result<()> {
    let app_cli = App::new("Cobalt")
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
        .arg(Arg::with_name("dump")
                 .long("dump")
                 .possible_values(&Dump::variants())
                 .help("Dump the specified internal state")
                 .global(true)
                 .multiple(true)
                 .takes_value(true))
        .subcommand(SubCommand::with_name("init")
                        .about("create a new cobalt project")
                        .arg(Arg::with_name("DIRECTORY")
                                 .help("Suppress all output")
                                 .default_value("./")
                                 .index(1)))
        .subcommand(SubCommand::with_name("new")
                        .about("Create a new post or page")
                        .arg(Arg::with_name("FILETYPE")
                                 .help("Type of file to create eg post or page")
                                 .default_value("post")
                                 .takes_value(true))
                        .arg(Arg::with_name("FILENAME")
                                 .help("File to create")
                                 .default_value_if("FILETYPE",
                                                   Some("page"),
                                                   "new_page.md")
                                 .default_value("new_post.md")
                                 .takes_value(true)))
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
        .subcommand(SubCommand::with_name("list-syntax-themes").about("list available themes"))
        .subcommand(SubCommand::with_name("list-syntaxes").about("list supported syntaxes"));

    let global_matches = app_cli.get_matches();

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

    match matches
              .value_of("log-level")
              .or_else(|| global_matches.value_of("log-level")) {
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

    let config_path = matches
        .value_of("config")
        .or_else(|| global_matches.value_of("config"))
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

    config.source = matches
        .value_of("source")
        .or_else(|| global_matches.value_of("source"))
        .map(str::to_string)
        .unwrap_or(config.source);

    config.dest = matches
        .value_of("destination")
        .or_else(|| global_matches.value_of("destination"))
        .map(str::to_string)
        .unwrap_or(config.dest);

    config.layouts = matches
        .value_of("layouts")
        .or_else(|| global_matches.value_of("layouts"))
        .map(str::to_string)
        .unwrap_or(config.layouts);

    config.posts = matches
        .value_of("posts")
        .or_else(|| global_matches.value_of("posts"))
        .map(str::to_string)
        .unwrap_or(config.posts);

    config.include_drafts = matches.is_present("drafts");

    if global_matches.is_present("dump") {
        let mut dump = values_t!(global_matches, "dump", Dump)?;
        config.dump.append(&mut dump);
        info!("Setting: {:?}", config.dump);
    }
    if matches.is_present("dump") {
        let mut dump = values_t!(matches, "dump", Dump)?;
        config.dump.append(&mut dump);
        info!("Setting: {:?}", config.dump);
    }

    match command {
        "init" => new::init_command(config, matches)?,
        "new" => new::new_command(config, matches)?,
        "build" => build::build_command(config, matches)?,
        "clean" => build::clean_command(config, matches)?,
        "serve" => serve::serve_command(config, matches)?,
        "watch" => serve::watch_command(config, matches)?,
        "import" => build::import_command(config, matches)?,
        "list-syntax-themes" => {
            for name in list_syntax_themes() {
                println!("{}", name);
            }
        }
        "list-syntaxes" => {
            for name in list_syntaxes() {
                println!("{}", name);
            }
        }
        _ => {
            bail!(global_matches.usage());
        }
    };

    Ok(())
}
