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
        .arg(clap::Arg::with_name("import")
                 .short("i")
                 .long("import")
                 .help("Import after build to gh-pages branch")
                 .takes_value(false))
        .arg(clap::Arg::with_name("branch")
                 .short("b")
                 .long("branch")
                 .value_name("BRANCH")
                 .help("Branch that will be used to import the site to")
                 .default_value("gh-pages")
                 .takes_value(true))
        .arg(clap::Arg::with_name("message")
                 .short("m")
                 .long("message")
                 .value_name("COMMIT-MESSAGE")
                 .help("Commit message that will be used on import")
                 .default_value("cobalt site import")
                 .takes_value(true))
}

pub fn build_command(matches: &clap::ArgMatches) -> Result<()> {
    let config = args::get_config(matches)?;
    let config = config.build()?;

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
    info!("Building from {:?} into {:?}", config.source, config.dest);
    cobalt::build(config)?;

    Ok(())
}

pub fn clean_command_args() -> clap::App<'static, 'static> {
    clap::SubCommand::with_name("clean")
        .about("cleans directory set as destination")
        .args(&args::get_config_args())
}

pub fn clean_command(matches: &clap::ArgMatches) -> Result<()> {
    let config = args::get_config(matches)?;
    let config = config.build()?;

    let cwd = env::current_dir().unwrap_or_else(|_| path::PathBuf::new());
    let destdir = config
        .dest
        .canonicalize()
        .unwrap_or_else(|_| path::PathBuf::new());
    if cwd == destdir {
        bail!("Destination directory is same as current directory. \
                       Cancelling the operation");
    }

    fs::remove_dir_all(&config.dest)?;

    info!("directory \"{:?}\" removed", &config.dest);

    Ok(())
}

pub fn import_command_args() -> clap::App<'static, 'static> {
    clap::SubCommand::with_name("import")
        .about("moves the contents of the dest folder to the gh-pages branch")
        .args(&args::get_config_args())
        .arg(clap::Arg::with_name("branch")
                 .short("b")
                 .long("branch")
                 .value_name("BRANCH")
                 .help("Branch that will be used to import the site to")
                 .default_value("gh-pages")
                 .takes_value(true))
        .arg(clap::Arg::with_name("message")
                 .short("m")
                 .long("message")
                 .value_name("COMMIT-MESSAGE")
                 .help("Commit message that will be used on import")
                 .default_value("cobalt site import")
                 .takes_value(true))
}

pub fn import_command(matches: &clap::ArgMatches) -> Result<()> {
    let config = args::get_config(matches)?;
    let config = config.build()?;

    let branch = matches.value_of("branch").unwrap().to_string();
    let message = matches.value_of("message").unwrap().to_string();
    import(&config, &branch, &message)?;

    Ok(())
}

fn import(config: &cobalt::Config, branch: &str, message: &str) -> Result<()> {
    info!("Importing {:?} to {}", config.dest, branch);

    if !config.dest.is_dir() {
        bail!("`{:?}` is not a directory", config.dest);
    }
    ghp::import_dir(&config.dest, branch, message)?;

    Ok(())
}
