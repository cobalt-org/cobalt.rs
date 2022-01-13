#![warn(warnings)]

#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate log;

mod args;
mod build;
mod debug;
mod error;
mod new;
#[cfg(feature = "serve")]
mod serve;

use std::alloc;

use clap::{AppSettings, Parser};

use crate::error::*;

#[global_allocator]
static GLOBAL: alloc::System = alloc::System;

/// Static site generator
#[derive(Clone, Debug, Parser)]
#[clap(global_setting = AppSettings::PropagateVersion)]
#[clap(version)]
struct Cli {
    #[clap(flatten)]
    pub logging: clap_verbosity_flag::Verbosity,

    #[clap(subcommand)]
    command: Command,
}

#[derive(Clone, Debug, PartialEq, Eq, Parser)]
enum Command {
    Init(new::InitArgs),
    New(new::NewArgs),
    Rename(new::RenameArgs),
    Publish(new::PublishArgs),
    Build(build::BuildArgs),
    Clean(build::CleanArgs),
    Import(build::ImportArgs),
    #[cfg(feature = "serve")]
    Serve(serve::ServeArgs),
    #[clap(subcommand)]
    Debug(debug::DebugCommands),
}

impl Cli {
    pub fn run(&self) -> Result<()> {
        let mut logging = self.logging.clone();
        logging.set_default(Some(log::Level::Info));
        if let Some(level) = logging.log_level() {
            let mut builder = args::get_logging(level)?;
            builder.init();
        }

        match &self.command {
            Command::Init(cmd) => cmd.run(),
            Command::New(cmd) => cmd.run(),
            Command::Rename(cmd) => cmd.run(),
            Command::Publish(cmd) => cmd.run(),
            Command::Build(cmd) => cmd.run(),
            Command::Clean(cmd) => cmd.run(),
            Command::Import(cmd) => cmd.run(),
            #[cfg(feature = "serve")]
            Command::Serve(cmd) => cmd.run(),
            Command::Debug(cmd) => cmd.run(),
        }
    }
}

fn main() -> std::result::Result<(), exitfailure::ExitFailure> {
    let cli = Cli::parse();
    cli.run()?;
    Ok(())
}

#[test]
fn verify_app() {
    use clap::IntoApp;
    Cli::into_app().debug_assert()
}
