use std::env;
use std::fs;

use crate::args;
use crate::error::Result;

/// Build the cobalt project at the source dir
#[derive(Clone, Debug, PartialEq, Eq, clap::Args)]
pub(crate) struct BuildArgs {
    /// Site destination folder [default: ./_site]
    #[arg(short, long, value_name = "DIR", help_heading = "Config")]
    destination: Option<std::path::PathBuf>,

    #[command(flatten, next_help_heading = "Config")]
    pub(crate) config: args::ConfigArgs,
}

impl BuildArgs {
    pub(crate) fn run(&self) -> Result<()> {
        let mut config = self.config.load_config()?;
        config.abs_dest = self
            .destination
            .as_deref()
            .map(|d| {
                fs::create_dir_all(d)?;
                dunce::canonicalize(d)
            })
            .transpose()?;

        let config = cobalt::cobalt_model::Config::from_config(config)?;

        build(config)?;
        log::info!("Build successful");

        Ok(())
    }
}

pub(crate) fn build(config: cobalt::Config) -> Result<()> {
    log::info!(
        "Building from `{}` into `{}`",
        config.source.display(),
        config.destination.display()
    );
    cobalt::build(config)?;

    Ok(())
}

/// Cleans `destination` directory
#[derive(Clone, Debug, PartialEq, Eq, clap::Args)]
pub(crate) struct CleanArgs {
    #[command(flatten, next_help_heading = "Config")]
    pub(crate) config: args::ConfigArgs,
}

impl CleanArgs {
    pub(crate) fn run(&self) -> Result<()> {
        let config = self.config.load_config()?;
        let config = cobalt::cobalt_model::Config::from_config(config)?;

        clean(&config)
    }
}

pub(crate) fn clean(config: &cobalt::Config) -> Result<()> {
    let cwd = env::current_dir().unwrap_or_default();
    let destdir = dunce::canonicalize(&config.destination);
    let destdir = match destdir {
        Ok(destdir) => destdir,
        Err(e) => {
            log::debug!("No `{}` to clean", config.destination.display());
            log::debug!("{e}");
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

    log::info!("directory `{}` removed", destdir.display());

    Ok(())
}
