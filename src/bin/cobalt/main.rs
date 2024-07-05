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

use clap::Parser;

use crate::error::Result;

#[global_allocator]
static GLOBAL: alloc::System = alloc::System;

/// Static site generator
#[derive(Clone, Debug, Parser)]
#[command(propagate_version = true)]
#[command(version)]
struct Cli {
    #[command(flatten)]
    pub(crate) logging: clap_verbosity_flag::Verbosity<clap_verbosity_flag::InfoLevel>,

    #[command(flatten)]
    pub(crate) color: colorchoice_clap::Color,

    #[command(subcommand)]
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
    #[cfg(feature = "serve")]
    Serve(serve::ServeArgs),
    #[command(subcommand)]
    Debug(debug::DebugCommands),
}

impl Cli {
    pub(crate) fn run(&self) -> Result<()> {
        self.color.write_global();
        let colored_stderr = !matches!(
            anstream::AutoStream::choice(&std::io::stderr()),
            anstream::ColorChoice::Never
        );
        args::init_logging(self.logging.clone(), colored_stderr);

        match &self.command {
            Command::Init(cmd) => cmd.run(),
            Command::New(cmd) => cmd.run(),
            Command::Rename(cmd) => cmd.run(),
            Command::Publish(cmd) => cmd.run(),
            Command::Build(cmd) => cmd.run(),
            Command::Clean(cmd) => cmd.run(),
            #[cfg(feature = "serve")]
            Command::Serve(cmd) => cmd.run(),
            Command::Debug(cmd) => cmd.run(),
        }
    }
}

fn main() -> Result<()> {
    human_panic::setup_panic!();
    let cli = Cli::parse();
    cli.run()?;
    Ok(())
}

#[test]
fn verify_app() {
    use clap::CommandFactory;
    Cli::command().debug_assert();
}
