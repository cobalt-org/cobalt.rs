use crate::args;
use crate::error::*;

/// Print site debug information
#[derive(Clone, Debug, PartialEq, Eq, clap::Subcommand)]
pub enum DebugCommands {
    /// Prints post-processed config
    Config {
        #[clap(flatten, next_help_heading = "CONFIG")]
        config: args::ConfigArgs,
    },

    /// Print syntax-highlight information
    #[clap(subcommand)]
    Highlight(HighlightCommands),

    /// Print files associated with a collection
    Files {
        /// Collection name
        collection: Option<String>,

        #[clap(flatten, next_help_heading = "CONFIG")]
        config: args::ConfigArgs,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, clap::Subcommand)]
pub enum HighlightCommands {
    Themes {
        #[clap(flatten, next_help_heading = "CONFIG")]
        config: args::ConfigArgs,
    },
    Syntaxes {
        #[clap(flatten, next_help_heading = "CONFIG")]
        config: args::ConfigArgs,
    },
}

impl DebugCommands {
    pub fn run(&self) -> Result<()> {
        match self {
            Self::Config { config } => {
                let config = config.load_config()?;
                let config = cobalt::cobalt_model::Config::from_config(config)?;
                println!("{}", config);
            }
            Self::Highlight(HighlightCommands::Themes { config }) => {
                let config = config.load_config()?;
                let config = cobalt::cobalt_model::Config::from_config(config)?;
                for name in config.syntax.themes() {
                    println!("{}", name);
                }
            }
            Self::Highlight(HighlightCommands::Syntaxes { config }) => {
                let config = config.load_config()?;
                let config = cobalt::cobalt_model::Config::from_config(config)?;
                for name in config.syntax.syntaxes() {
                    println!("{}", name);
                }
            }
            Self::Files { collection, config } => {
                let config = config.load_config()?;
                let config = cobalt::cobalt_model::Config::from_config(config)?;

                match collection.as_deref() {
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
        }

        Ok(())
    }
}
