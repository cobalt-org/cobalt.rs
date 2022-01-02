use crate::args;
use crate::error::*;

pub fn debug_command_args() -> clap::App<'static> {
    clap::App::new("debug")
        .about("Print site debug information")
        .subcommand(clap::App::new("config").about("Prints post-processed config"))
        .subcommand(
            clap::App::new("highlight")
                .about("Print syntax-highlight information")
                .subcommand(clap::App::new("themes"))
                .subcommand(clap::App::new("syntaxes")),
        )
        .subcommand(
            clap::App::new("files")
                .about("Print files associated with a collection")
                .args(args::get_config_args())
                .arg(
                    clap::Arg::new("COLLECTION")
                        .help("Collection name")
                        .index(1),
                ),
        )
}

pub fn debug_command(matches: &clap::ArgMatches) -> Result<()> {
    match matches.subcommand() {
        Some(("config", _)) => {
            let config = args::get_config(matches)?;
            let config = cobalt::cobalt_model::Config::from_config(config)?;
            println!("{}", config);
        }
        Some(("highlight", matches)) => match matches.subcommand() {
            Some(("themes", _)) => {
                for name in cobalt::list_syntax_themes() {
                    println!("{}", name);
                }
            }
            Some(("syntaxes", _)) => {
                for name in cobalt::list_syntaxes() {
                    println!("{}", name);
                }
            }
            _ => unreachable!("Unexpected subcommand"),
        },
        Some(("files", matches)) => {
            let config = args::get_config(matches)?;
            let config = cobalt::cobalt_model::Config::from_config(config)?;
            let collection = matches.value_of("COLLECTION");
            match collection {
                Some("assets") => {
                    failure::bail!("TODO Re-implement");
                }
                Some("pages") => {
                    failure::bail!("TODO Re-implement");
                }
                Some("posts") => {
                    failure::bail!("TODO Re-implement");
                }
                None => {
                    let source_files = cobalt_core::Source::new(
                        &config.source,
                        config.ignore.iter().map(|s| s.as_str()),
                    )?;
                    for path in source_files.iter() {
                        println!("{}", path.rel_path);
                    }
                }
                _ => {
                    failure::bail!("Collection is not yet supported");
                }
            }
        }
        _ => unreachable!("Unexpected subcommand"),
    }

    Ok(())
}
