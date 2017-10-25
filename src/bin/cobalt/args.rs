use std::env;

use clap;
use env_logger;
use log;

use cobalt;
use error::*;


pub fn get_config_args() -> Vec<clap::Arg<'static, 'static>> {
    [clap::Arg::with_name("config")
         .short("c")
         .long("config")
         .value_name("FILE")
         .help("Config file to use [default: .cobalt.yml]")
         .global(true)
         .takes_value(true),
     clap::Arg::with_name("destination")
         .short("d")
         .long("destination")
         .value_name("DIR")
         .help("Destination folder [default: ./]")
         .global(true)
         .takes_value(true),
     clap::Arg::with_name("drafts")
         .long("drafts")
         .help("Include drafts.")
         .global(true)
         .takes_value(false),
     clap::Arg::with_name("dump")
         .long("dump")
         .possible_values(&cobalt::Dump::variants())
         .help("Dump the specified internal state")
         .global(true)
         .multiple(true)
         .takes_value(true)]
        .to_vec()
}

pub fn get_config(global_matches: &clap::ArgMatches,
                  matches: &clap::ArgMatches)
                  -> Result<cobalt::ConfigBuilder> {
    let config_path = matches
        .value_of("config")
        .or_else(|| global_matches.value_of("config"));

    // Fetch config information if available
    let mut config = if let Some(config_path) = config_path {
        cobalt::ConfigBuilder::from_file(config_path)
            .chain_err(|| format!("Error reading config file {:?}", config_path))?
    } else {
        let cwd = env::current_dir().expect("How does this fail?");
        cobalt::ConfigBuilder::from_cwd(cwd)?
    };

    config.abs_dest = matches
        .value_of("destination")
        .or_else(|| global_matches.value_of("destination"))
        .map(str::to_string);

    config.include_drafts = matches.is_present("drafts");

    if global_matches.is_present("dump") {
        let mut dump = values_t!(global_matches, "dump", cobalt::Dump)?;
        config.dump.append(&mut dump);
        info!("Setting: {:?}", config.dump);
    }
    if matches.is_present("dump") {
        let mut dump = values_t!(matches, "dump", cobalt::Dump)?;
        config.dump.append(&mut dump);
        info!("Setting: {:?}", config.dump);
    }

    Ok(config)
}

pub fn get_logging_args() -> Vec<clap::Arg<'static, 'static>> {
    [clap::Arg::with_name("log-level")
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
         .takes_value(false)]
        .to_vec()
}

pub fn get_logging(global_matches: &clap::ArgMatches,
                   matches: &clap::ArgMatches)
                   -> Result<env_logger::LogBuilder> {
    let format = |record: &log::LogRecord| {
        let level = format!("[{}]", record.level()).to_lowercase();
        format!("{:8} {}", level, record.args())
    };

    let mut builder = env_logger::LogBuilder::new();
    builder.format(format);

    match matches
              .value_of("log-level")
              .or_else(|| global_matches.value_of("log-level")) {
        Some("error") => builder.filter(None, log::LogLevelFilter::Error),
        Some("warn") => builder.filter(None, log::LogLevelFilter::Warn),
        Some("debug") => builder.filter(None, log::LogLevelFilter::Debug),
        Some("trace") => builder.filter(None, log::LogLevelFilter::Trace),
        Some("off") => builder.filter(None, log::LogLevelFilter::Off),
        Some("info") | _ => builder.filter(None, log::LogLevelFilter::Info),
    };

    if matches.is_present("trace") {
        builder.filter(None, log::LogLevelFilter::Trace);
    }

    if matches.is_present("silent") {
        builder.filter(None, log::LogLevelFilter::Off);
    }

    Ok(builder)
}
