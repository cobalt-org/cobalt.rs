use std::env;
use std::io::Write;
use std::path;

use clap;
use env_logger;
use log;

use cobalt;
use error::*;

pub fn get_config_args() -> Vec<clap::Arg<'static, 'static>> {
    [
        clap::Arg::with_name("config")
            .short("c")
            .long("config")
            .value_name("FILE")
            .help("Config file to use [default: _cobalt.yml]")
            .takes_value(true),
        clap::Arg::with_name("destination")
            .short("d")
            .long("destination")
            .value_name("DIR")
            .help("Site destination folder [default: ./]")
            .takes_value(true),
        clap::Arg::with_name("drafts")
            .long("drafts")
            .help("Include drafts.")
            .takes_value(false),
        clap::Arg::with_name("no-drafts")
            .long("no-drafts")
            .help("Ignore drafts.")
            .conflicts_with("drafts")
            .takes_value(false),
    ].to_vec()
}

pub fn get_config(matches: &clap::ArgMatches) -> Result<cobalt::ConfigBuilder> {
    let config_path = matches.value_of("config");

    // Fetch config information if available
    let mut config = if let Some(config_path) = config_path {
        cobalt::ConfigBuilder::from_file(config_path)
            .chain_err(|| format!("Error reading config file {:?}", config_path))?
    } else {
        let cwd = env::current_dir().expect("How does this fail?");
        cobalt::ConfigBuilder::from_cwd(cwd)?
    };

    config.abs_dest = matches.value_of("destination").map(path::PathBuf::from);

    if matches.is_present("drafts") {
        config.include_drafts = true;
    }
    if matches.is_present("no-drafts") {
        config.include_drafts = false;
    }

    Ok(config)
}

pub fn get_logging_args() -> Vec<clap::Arg<'static, 'static>> {
    [
        clap::Arg::with_name("log-level")
            .short("L")
            .long("log-level")
            .possible_values(&["error", "warn", "info", "debug", "trace", "off"])
            .help("Log level [default: info]")
            .global(true)
            .takes_value(true),
        clap::Arg::with_name("trace")
            .long("trace")
            .help("Log ultra-verbose (trace level) information")
            .global(true)
            .takes_value(false),
        clap::Arg::with_name("silent")
            .long("silent")
            .help("Suppress all output")
            .global(true)
            .takes_value(false),
    ].to_vec()
}

pub fn get_logging(
    global_matches: &clap::ArgMatches,
    matches: &clap::ArgMatches,
) -> Result<env_logger::Builder> {
    let mut builder = env_logger::Builder::new();

    let level = if matches.is_present("trace") {
        log::LevelFilter::Trace
    } else if matches.is_present("silent") {
        log::LevelFilter::Off
    } else {
        match matches
            .value_of("log-level")
            .or_else(|| global_matches.value_of("log-level"))
        {
            Some("error") => log::LevelFilter::Error,
            Some("warn") => log::LevelFilter::Warn,
            Some("debug") => log::LevelFilter::Debug,
            Some("trace") => log::LevelFilter::Trace,
            Some("off") => log::LevelFilter::Off,
            Some("info") | _ => log::LevelFilter::Info,
        }
    };
    builder.filter(None, level);

    if level == log::LevelFilter::Trace {
        builder.default_format_timestamp(false);
    } else {
        builder.format(|f, record| {
            writeln!(
                f,
                "[{}] {}",
                record.level().to_string().to_lowercase(),
                record.args()
            )
        });
    }

    Ok(builder)
}
