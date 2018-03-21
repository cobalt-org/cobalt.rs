#![warn(warnings)]

extern crate cobalt;
extern crate env_logger;
extern crate ghp;
extern crate hyper;
extern crate notify;
extern crate serde_yaml;

#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate error_chain;

#[macro_use]
extern crate clap;

#[macro_use]
extern crate log;

mod args;
mod build;
mod error;
mod debug;
mod jekyll;
mod new;
mod serve;

use clap::{App, AppSettings};

use error::*;

quick_main!(run);

fn run() -> Result<()> {
    let app_cli = App::new("Cobalt")
        .version(crate_version!())
        .author("Benny Klotz <r3qnbenni@gmail.com>, Johann Hofmann")
        .about("A static site generator written in Rust.")
        .setting(AppSettings::SubcommandRequired)
        .setting(AppSettings::GlobalVersion)
        .args(&args::get_logging_args())
        .subcommand(new::init_command_args())
        .subcommand(new::new_command_args())
        .subcommand(new::rename_command_args())
        .subcommand(new::publish_command_args())
        .subcommand(build::build_command_args())
        .subcommand(build::clean_command_args())
        .subcommand(serve::serve_command_args())
        .subcommand(build::import_command_args())
        .subcommand(jekyll::convert_command_args())
        .subcommand(debug::debug_command_args());

    let global_matches = app_cli.get_matches();

    let (command, matches) = match global_matches.subcommand() {
        (command, Some(matches)) => (command, matches),
        (_, None) => unreachable!(),
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
        "serve" => serve::serve_command(matches),
        "import" => build::import_command(matches),
        "debug" => debug::debug_command(matches),
        "convert-jekyll" => jekyll::convert_command(matches),
        _ => {
            bail!(global_matches.usage());
        }
    }.chain_err(|| format!("{} command failed", command))?;

    Ok(())
}
