use crate::args;
use crate::error::*;

pub fn debug_command_args() -> clap::App<'static, 'static> {
    clap::SubCommand::with_name("debug")
        .about("Print site debug information")
        .subcommand(clap::SubCommand::with_name("config").about("Prints post-processed config"))
        .subcommand(
            clap::SubCommand::with_name("highlight")
                .about("Print syntax-highlight information")
                .subcommand(clap::SubCommand::with_name("themes"))
                .subcommand(clap::SubCommand::with_name("syntaxes")),
        )
        .subcommand(
            clap::SubCommand::with_name("files")
                .about("Print files associated with a collection")
                .args(&args::get_config_args())
                .arg(
                    clap::Arg::with_name("COLLECTION")
                        .help("Collection name")
                        .index(1),
                ),
        )
}

pub fn debug_command(matches: &clap::ArgMatches<'_>) -> Result<()> {
    match matches.subcommand() {
        ("config", _) => {
            let config = args::get_config(matches)?;
            let config = cobalt::cobalt_model::Config::from_config(config)?;
            println!("{}", config);
        }
        ("highlight", Some(matches)) => match matches.subcommand() {
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
            _ => failure::bail!(matches.usage().to_owned()),
        },
        ("files", Some(matches)) => {
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
        _ => failure::bail!(matches.usage().to_owned()),
    }

    Ok(())
}
