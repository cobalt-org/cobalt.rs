use std::env;
use std::fs;
use std::path;
use std::process;

use clap;
use cobalt;
use ghp;

use error::*;

pub fn build_command(config: cobalt::Config, matches: &clap::ArgMatches) -> Result<()> {
    build(&config)?;
    info!("Build successful");

    if matches.is_present("import") {
        let branch = matches.value_of("branch").unwrap().to_string();
        let message = matches.value_of("message").unwrap().to_string();
        import(&config, &branch, &message)?
    }

    Ok(())
}

pub fn build(config: &cobalt::Config) -> Result<()> {
    info!("Building from {} into {}", config.source, config.dest);
    cobalt::build(config)?;

    Ok(())
}

pub fn clean_command(config: cobalt::Config, _matches: &clap::ArgMatches) -> Result<()> {
    let cwd = env::current_dir().unwrap_or_else(|_| path::PathBuf::new());
    let destdir = path::PathBuf::from(&config.dest);
    let destdir = fs::canonicalize(destdir).unwrap_or_else(|_| path::PathBuf::new());
    if cwd == destdir {
        error!("Destination directory is same as current directory. \
                       Cancelling the operation");
        process::exit(1);
    }

    fs::remove_dir_all(&config.dest)?;

    info!("directory \"{}\" removed", &config.dest);

    Ok(())
}

pub fn import_command(config: cobalt::Config, matches: &clap::ArgMatches) -> Result<()> {
    let branch = matches.value_of("branch").unwrap().to_string();
    let message = matches.value_of("message").unwrap().to_string();
    import(&config, &branch, &message)?;

    Ok(())
}

fn import(config: &cobalt::Config, branch: &str, message: &str) -> Result<()> {
    info!("Importing {} to {}", config.dest, branch);

    let meta = fs::metadata(&config.dest)?;

    if !meta.is_dir() {
        bail!("`{}` is not a directory", config.dest);
    }
    ghp::import_dir(&config.dest, branch, message)?;

    Ok(())
}
