use crate::args;
use crate::error::Result;
#[cfg(feature = "syntax-highlight")]
use std::path::PathBuf;

/// Print site debug information
#[derive(Clone, Debug, PartialEq, Eq, clap::Subcommand)]
pub(crate) enum DebugCommands {
    /// Prints post-processed config
    Config {
        #[command(flatten, next_help_heading = "Config")]
        config: args::ConfigArgs,
    },

    /// Print syntax-highlight information
    #[command(subcommand)]
    Highlight(HighlightCommands),

    /// Print files associated with a collection
    Files {
        /// Collection name
        collection: Option<String>,

        #[command(flatten, next_help_heading = "Config")]
        config: args::ConfigArgs,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, clap::Subcommand)]
pub(crate) enum HighlightCommands {
    Themes {
        #[command(flatten, next_help_heading = "Config")]
        config: args::ConfigArgs,
    },
    Syntaxes {
        #[command(flatten, next_help_heading = "Config")]
        config: args::ConfigArgs,
    },

    /// Save the css file for a theme
    #[cfg(feature = "syntax-highlight")]
    SaveThemeCss {
        #[command(flatten, next_help_heading = "Config")]
        config: args::ConfigArgs,

        /// Name of the theme to generate a css file for
        name: String,

        /// Path of the css file
        path: PathBuf,
    },
}

impl DebugCommands {
    pub(crate) fn run(&self) -> Result<()> {
        match self {
            Self::Config { config } => {
                let config = config.load_config()?;
                let config = cobalt::cobalt_model::Config::from_config(config)?;
                println!("{config}");
            }
            Self::Highlight(HighlightCommands::Themes { config }) => {
                let config = config.load_config()?;
                let config = cobalt::cobalt_model::Config::from_config(config)?;
                for name in config.syntax.themes() {
                    println!("{name}");
                }
            }
            Self::Highlight(HighlightCommands::Syntaxes { config }) => {
                let config = config.load_config()?;
                let config = cobalt::cobalt_model::Config::from_config(config)?;
                for name in config.syntax.syntaxes() {
                    println!("{name}");
                }
            }
            #[cfg(feature = "syntax-highlight")]
            Self::Highlight(HighlightCommands::SaveThemeCss { config, name, path }) => {
                let config = config.load_config()?;
                let config = cobalt::cobalt_model::Config::from_config(config)?;
                if name == engarde::Syntax::css_theme_name() || !config.syntax.has_theme(name) {
                    return Err(anyhow::anyhow!("Unknown theme: {name}"));
                }

                let css = config.syntax.css_for_theme(name);

                std::fs::write(path, css)?;

                println!(
                    "CSS theme '{name}' successfully saved to: {}",
                    path.display()
                );
            }
            Self::Files { collection, config } => {
                let config = config.load_config()?;
                let config = cobalt::cobalt_model::Config::from_config(config)?;

                match collection.as_deref() {
                    Some("assets") => {
                        anyhow::bail!("TODO Re-implement");
                    }
                    Some("pages") => {
                        anyhow::bail!("TODO Re-implement");
                    }
                    Some("posts") => {
                        anyhow::bail!("TODO Re-implement");
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
                        anyhow::bail!("Collection is not yet supported");
                    }
                }
            }
        }

        Ok(())
    }
}
