// Deny warnings, except in `dev` mode

// To update this list
// 1. Run `rustc -W help`
// 2. Grab all `default=warn` warnings
// 3. Paste them here, deleting `warnings`, and any with `deprecated` in the name
#![deny(const_err,
        dead_code,
        illegal_floating_point_literal_pattern,
        improper_ctypes,
        non_camel_case_types,
        non_shorthand_field_patterns,
        non_snake_case,
        non_upper_case_globals,
        no_mangle_generic_items,
        overflowing_literals,
        path_statements,
        patterns_in_fns_without_body,
        plugin_as_library,
        private_in_public,
        private_no_mangle_fns,
        private_no_mangle_statics,
        renamed_and_removed_lints,
        stable_features,
        unconditional_recursion,
        unions_with_drop_fields,
        unknown_lints,
        unreachable_code,
        unreachable_patterns,
        unused_allocation,
        unused_assignments,
        unused_attributes,
        unused_comparisons,
        unused_features,
        unused_imports,
        unused_macros,
        unused_must_use,
        unused_mut,
        unused_parens,
        unused_unsafe,
        unused_variables,
        while_true)]
// This list is select `allow` warnings
#![deny(trivial_casts,
       trivial_numeric_casts,
       unused_extern_crates,
       unused_import_braces)]
#![cfg_attr(feature="cargo-clippy", allow(
        cyclomatic_complexity,
        needless_pass_by_value))]
#![cfg_attr(feature="dev", warn(warnings))]

extern crate cobalt;
extern crate env_logger;
extern crate ghp;
extern crate hyper;
extern crate notify;
extern crate regex;
extern crate serde_yaml;

#[macro_use]
extern crate error_chain;

#[macro_use]
extern crate clap;

#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate log;

mod args;
mod build;
mod error;
mod debug;
mod jekyll;
mod migrate;
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
        .subcommand(new::publish_command_args())
        .subcommand(build::build_command_args())
        .subcommand(build::clean_command_args())
        .subcommand(serve::serve_command_args())
        .subcommand(build::import_command_args())
        .subcommand(migrate::migrate_command_args())
        .subcommand(jekyll::convert_command_args())
        .subcommand(debug::debug_command_args());

    let global_matches = app_cli.get_matches();

    let (command, matches) = match global_matches.subcommand() {
        (command, Some(matches)) => (command, matches),
        (_, None) => unreachable!(),
    };

    let mut builder = args::get_logging(&global_matches, matches)?;
    builder.init().unwrap();

    match command {
        "init" => new::init_command(matches),
        "new" => new::new_command(matches),
        "publish" => new::publish_command(matches),
        "build" => build::build_command(matches),
        "clean" => build::clean_command(matches),
        "serve" => serve::serve_command(matches),
        "import" => build::import_command(matches),
        "debug" => debug::debug_command(matches),
        "migrate" => migrate::migrate_command(matches),
        "convert-jekyll" => jekyll::convert_command(matches),
        _ => {
            bail!(global_matches.usage());
        }
    }.chain_err(|| format!("{} command failed", command))?;

    Ok(())
}
