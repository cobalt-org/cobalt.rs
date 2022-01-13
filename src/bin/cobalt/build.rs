use std::env;
use std::fs;
use std::path;

use crate::args;
use crate::error::*;

/// Build the cobalt project at the source dir
#[derive(Clone, Debug, PartialEq, Eq, clap::Args)]
pub struct BuildArgs {
    #[clap(flatten, help_heading = "CONFIG")]
    pub config: args::ConfigArgs,
}

impl BuildArgs {
    pub fn run(&self) -> Result<()> {
        let config = self.config.load_config()?;
        let config = cobalt::cobalt_model::Config::from_config(config)?;

        build(config)?;
        info!("Build successful");

        Ok(())
    }
}

pub fn build(config: cobalt::Config) -> Result<()> {
    info!(
        "Building from {:?} into {:?}",
        config.source, config.destination
    );
    cobalt::build(config)?;

    Ok(())
}

/// Cleans `destination` directory
#[derive(Clone, Debug, PartialEq, Eq, clap::Args)]
pub struct CleanArgs {
    #[clap(flatten, help_heading = "CONFIG")]
    pub config: args::ConfigArgs,
}

impl CleanArgs {
    pub fn run(&self) -> Result<()> {
        let config = self.config.load_config()?;
        let config = cobalt::cobalt_model::Config::from_config(config)?;

        clean(&config)
    }
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
        failure::bail!(
            "Attempting to delete current directory ({:?}), \
             Cancelling the operation",
            destdir
        );
    }

    fs::remove_dir_all(&destdir)?;

    info!("directory \"{:?}\" removed", &destdir);

    Ok(())
}

/// Moves the contents of the dest folder to the gh-pages branch
#[derive(Clone, Debug, PartialEq, Eq, clap::Args)]
pub struct ImportArgs {
    /// Branch that will be used to import the site to
    #[clap(short, long, default_value = "gh-pages")]
    pub branch: String,

    /// Commit message that will be used on import
    #[clap(
        short,
        long,
        value_name = "COMMIT-MESSAGE",
        default_value = "cobalt site import"
    )]
    pub message: String,

    #[clap(flatten, help_heading = "CONFIG")]
    pub config: args::ConfigArgs,
}

impl ImportArgs {
    pub fn run(&self) -> Result<()> {
        let config = self.config.load_config()?;
        let config = cobalt::cobalt_model::Config::from_config(config)?;

        clean(&config)?;
        build(config.clone())?;

        import(&config, &self.branch, &self.message)?;

        Ok(())
    }
}

fn import(config: &cobalt::Config, branch: &str, message: &str) -> Result<()> {
    info!("Importing {:?} to {}", config.destination, branch);

    if !config.destination.is_dir() {
        failure::bail!("`{:?}` is not a directory", config.destination);
    }
    ghp::import_dir(&config.destination, branch, message)?;

    Ok(())
}
