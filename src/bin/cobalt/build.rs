use std::env;
use std::fs;
use std::path;

use clap;
use cobalt;
use ghp;

use args;
use error::*;

pub fn build_command_args() -> clap::App<'static, 'static> {
    clap::SubCommand::with_name("build")
        .about("build the cobalt project at the source dir")
        .args(&args::get_config_args())
}

pub fn build_command(matches: &clap::ArgMatches) -> Result<()> {
    let config = args::get_config(matches)?;
    let config = config.build()?;

    build(config.clone())?;
    info!("Build successful");

    Ok(())
}

pub fn build(config: cobalt::Config) -> Result<()> {
    info!(
        "Building from {:?} into {:?}",
        config.source, config.destination
    );
    cobalt::build(config)?;

    Ok(())
}

pub fn clean_command_args() -> clap::App<'static, 'static> {
    clap::SubCommand::with_name("clean")
        .about("cleans `destination` directory")
        .args(&args::get_config_args())
}

pub fn clean_command(matches: &clap::ArgMatches) -> Result<()> {
    let config = args::get_config(matches)?;
    let config = config.build()?;

    clean(&config)
}

pub fn clean(config: &cobalt::Config) -> Result<()> {
    let cwd = env::current_dir().unwrap_or_else(|_| path::PathBuf::new());
    let destdir = config.destination.canonicalize();
    let destdir = match destdir {
        Ok(destdir) => destdir,
        Err(e) => {
            debug!("No \"{:?}\" to clean", &config.destination);
            debug!("{}", e);
            return Ok(());
        }
    };
    if cwd.starts_with(&destdir) {
        bail!(
            "Attempting to delete current directory ({:?}), \
             Cancelling the operation",
            destdir
        );
    }

    fs::remove_dir_all(&destdir)?;

    info!("directory \"{:?}\" removed", &destdir);

    Ok(())
}

pub fn import_command_args() -> clap::App<'static, 'static> {
    clap::SubCommand::with_name("import")
        .about("moves the contents of the dest folder to the gh-pages branch")
        .args(&args::get_config_args())
        .arg(
            clap::Arg::with_name("branch")
                .short("b")
                .long("branch")
                .value_name("BRANCH")
                .help("Branch that will be used to import the site to")
                .default_value("gh-pages")
                .takes_value(true),
        )
        .arg(
            clap::Arg::with_name("message")
                .short("m")
                .long("message")
                .value_name("COMMIT-MESSAGE")
                .help("Commit message that will be used on import")
                .default_value("cobalt site import")
                .takes_value(true),
        )
}

pub fn import_command(matches: &clap::ArgMatches) -> Result<()> {
    let config = args::get_config(matches)?;
    let config = config.build()?;

    clean(&config)?;
    build(config.clone())?;

    let branch = matches.value_of("branch").unwrap().to_string();
    let message = matches.value_of("message").unwrap().to_string();
    import(&config, &branch, &message)?;

    Ok(())
}

fn import(config: &cobalt::Config, branch: &str, message: &str) -> Result<()> {
    info!("Importing {:?} to {}", config.destination, branch);

    if !config.destination.is_dir() {
        bail!("`{:?}` is not a directory", config.destination);
    }
    ghp::import_dir(&config.destination, branch, message)?;

    Ok(())
}
