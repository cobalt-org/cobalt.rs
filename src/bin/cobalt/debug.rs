use clap;
use cobalt;

use error::*;

pub fn debug_command_args() -> clap::App<'static, 'static> {
    clap::SubCommand::with_name("debug")
        .about("Print site debug information")
        .subcommand(clap::SubCommand::with_name("highlight")
                        .about("Print syntax-highlight information")
                        .subcommand(clap::SubCommand::with_name("themes"))
                        .subcommand(clap::SubCommand::with_name("syntaxes")))
}

pub fn debug_command(matches: &clap::ArgMatches) -> Result<()> {
    match matches.subcommand() {
        ("highlight", Some(matches)) => {
            match matches.subcommand() {
                ("themes", _) => {
                    for name in cobalt::list_syntax_themes() {
                        println!("{}", name);
                    }
                }
                ("syntaxes", _) => {
                    for name in cobalt::list_syntaxes() {
                        println!("{}", name);
                    }
                }
                _ => bail!(matches.usage()),
            }
        }
        _ => bail!(matches.usage()),
    }

    Ok(())
}
