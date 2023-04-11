use std::env;
use std::fs;

use crate::args;
use crate::error::*;

/// Build the cobalt project at the source dir
#[derive(Clone, Debug, PartialEq, Eq, clap::Args)]
pub struct BuildArgs {
    #[command(flatten, next_help_heading = "Config")]
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
        "Building from `{}` into `{}`",
        config.source.display(),
        config.destination.display()
    );
    cobalt::build(config)?;

    Ok(())
}

/// Cleans `destination` directory
#[derive(Clone, Debug, PartialEq, Eq, clap::Args)]
pub struct CleanArgs {
    #[command(flatten, next_help_heading = "Config")]
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
    let cwd = env::current_dir().unwrap_or_default();
    let destdir = dunce::canonicalize(&config.destination);
    let destdir = match destdir {
        Ok(destdir) => destdir,
        Err(e) => {
            debug!("No `{}` to clean", config.destination.display());
            debug!("{}", e);
            return Ok(());
        }
    };
    if cwd.starts_with(&destdir) {
        anyhow::bail!(
            "Attempting to delete current directory ({:?}), \
             Cancelling the operation",
            destdir
        );
    }

    fs::remove_dir_all(&destdir)?;

    info!("directory `{}` removed", destdir.display());

    Ok(())
}
