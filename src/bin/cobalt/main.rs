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
#![cfg_attr(feature="cargo-clippy", allow(
        cyclomatic_complexity,
        needless_pass_by_value))]
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

use std::env;

use clap::{Arg, App, SubCommand, AppSettings};
use cobalt::{ConfigBuilder, Dump, jekyll};
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
        .arg(Arg::with_name("destination")
                 .short("d")
                 .long("destination")
                 .value_name("DIR")
                 .help("Destination folder [default: ./]")
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
                                 .default_value_if("FILETYPE", Some("page"), "new_page.md")
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
        .subcommand(SubCommand::with_name("list-syntaxes").about("list supported syntaxes"))
        .subcommand(SubCommand::with_name("convert-jekyll")
                        .about("convert jekyll website to cobalt")
                        .arg(Arg::with_name("jksrc")
                                 .long("jksrc")
                                 .value_name("JEKYLL-FILE-OR-DIR")
                                 .help("Jekyll posts' directory")
                                 .required(true)
                                 .takes_value(true))
                        .arg(Arg::with_name("jkdst")
                                 .long("jkdst")
                                 .value_name("DIR")
                                 .help("Output dir of converted posts")
                                 .takes_value(true)
                                 .default_value("./posts")));

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
        .or_else(|| global_matches.value_of("config"));

    // Fetch config information if available
    let mut config = if let Some(config_path) = config_path {
        ConfigBuilder::from_file(config_path)
            .chain_err(|| format!("Error reading config file {:?}", config_path))?
    } else {
        let cwd = env::current_dir().expect("How does this fail?");
        ConfigBuilder::from_cwd(cwd)?
    };

    config.dest = matches
        .value_of("destination")
        .or_else(|| global_matches.value_of("destination"))
        .map(str::to_string)
        .unwrap_or(config.dest);

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

    let config = config.build()?;

    match command {
        "init" => new::init_command(config, matches),
        "new" => new::new_command(config, matches),
        "build" => build::build_command(config, matches),
        "clean" => build::clean_command(config, matches),
        "serve" => serve::serve_command(config, matches),
        "watch" => serve::watch_command(config, matches),
        "import" => build::import_command(config, matches),
        "list-syntax-themes" => {
            for name in list_syntax_themes() {
                println!("{}", name);
            }
            Ok(())
        }
        "list-syntaxes" => {
            for name in list_syntaxes() {
                println!("{}", name);
            }
            Ok(())
        }
        "convert-jekyll" => {
            let source = matches.value_of("jksrc").unwrap().to_string();
            let dest = matches.value_of("jkdst").unwrap().to_string();
            jekyll::jk_document::convert_from_jk(std::path::Path::new(&source),
                                                 std::path::Path::new(&dest))
                .chain_err(|| "Jekyll conversion failed.")
        }
        _ => {
            bail!(global_matches.usage());
        }
    }.chain_err(|| format!("{} command failed", command))?;

    Ok(())
}
