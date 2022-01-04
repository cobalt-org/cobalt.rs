#![warn(warnings)]

#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate clap;

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

use clap::{App, AppSettings};
use failure::ResultExt;

use crate::error::*;

#[global_allocator]
static GLOBAL: alloc::System = alloc::System;

fn main() -> std::result::Result<(), exitfailure::ExitFailure> {
    run()?;
    Ok(())
}

fn cli() -> App<'static> {
    let app_cli = App::new("Cobalt")
        .version(crate_version!())
        .author("Benny Klotz <r3qnbenni@gmail.com>, Johann Hofmann")
        .about("A static site generator written in Rust.")
        .setting(AppSettings::SubcommandRequired)
        .setting(AppSettings::PropagateVersion)
        .args(&args::get_logging_args())
        .subcommand(new::init_command_args())
        .subcommand(new::new_command_args())
        .subcommand(new::rename_command_args())
        .subcommand(new::publish_command_args())
        .subcommand(build::build_command_args())
        .subcommand(build::clean_command_args())
        .subcommand(build::import_command_args())
        .subcommand(debug::debug_command_args());
    #[cfg(feature = "serve")]
    let app_cli = app_cli.subcommand(serve::serve_command_args());
    app_cli
}

fn run() -> Result<()> {
    let app_cli = cli();
    let global_matches = app_cli.get_matches();

    let (command, matches) = match global_matches.subcommand() {
        Some((command, matches)) => (command, matches),
        None => unreachable!(),
    };

    let mut builder = args::get_logging(&global_matches, matches)?;
    builder.init();

    match command {
        "init" => new::init_command(matches),
        "new" => new::new_command(matches),
        "rename" => new::rename_command(matches),
        "publish" => new::publish_command(matches),
        "build" => build::build_command(matches),
        "clean" => build::clean_command(matches),
        #[cfg(feature = "serve")]
        "serve" => serve::serve_command(matches),
        "import" => build::import_command(matches),
        "debug" => debug::debug_command(matches),
        _ => unreachable!("Unexpected subcommand"),
    }
    .with_context(|_| failure::format_err!("{} command failed", command))?;

    Ok(())
}

#[test]
fn verify_app() {
    cli().debug_assert()
}
