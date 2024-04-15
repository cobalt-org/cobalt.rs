use std::env;
use std::io::Write;
use std::path;

use anyhow::Context as _;

use crate::error::*;

#[derive(Clone, Debug, PartialEq, Eq, clap::Args)]
pub struct ConfigArgs {
    /// Config file to use [default: _cobalt.yml]
    #[arg(short, long, value_name = "FILE")]
    config: Option<path::PathBuf>,

    /// Site destination folder [default: ./]
    #[arg(short, long, value_name = "DIR")]
    destination: Option<path::PathBuf>,

    /// Include drafts.
    #[arg(long)]
    drafts: bool,

    /// Ignore drafts.
    #[arg(long, conflicts_with = "drafts")]
    no_drafts: bool,
}

impl ConfigArgs {
    pub fn load_config(&self) -> Result<cobalt_config::Config> {
        let config_path = self.config.as_deref();

        // Fetch config information if available
        let mut config = if let Some(config_path) = config_path {
            cobalt_config::Config::from_file(config_path).with_context(|| {
                anyhow::format_err!("Error reading config file {}", config_path.display())
            })?
        } else {
            let cwd = env::current_dir().unwrap_or_default();
            cobalt_config::Config::from_cwd(cwd)?
        };

        config.abs_dest = self
            .destination
            .as_deref()
            .map(|d| {
                std::fs::create_dir_all(d)?;
                dunce::canonicalize(d)
            })
            .transpose()?;

        if let Some(drafts) = self.drafts() {
            config.include_drafts = drafts;
        }

        Ok(config)
    }

    pub fn drafts(&self) -> Option<bool> {
        resolve_bool_arg(self.drafts, self.no_drafts)
    }
}

fn resolve_bool_arg(yes: bool, no: bool) -> Option<bool> {
    match (yes, no) {
        (true, false) => Some(true),
        (false, true) => Some(false),
        (false, false) => None,
        (_, _) => unreachable!("clap should make this impossible"),
    }
}

pub fn init_logging(
    level: clap_verbosity_flag::Verbosity<clap_verbosity_flag::InfoLevel>,
    colored: bool,
) {
    if let Some(level) = level.log_level() {
        let palette = if colored {
            Palette::colored()
        } else {
            Palette::plain()
        };

        let mut builder = env_logger::Builder::new();
        builder.write_style(if colored {
            env_logger::WriteStyle::Always
        } else {
            env_logger::WriteStyle::Never
        });

        builder.filter(None, level.to_level_filter());

        if level == log::LevelFilter::Trace {
            builder.format_timestamp_secs();
        } else {
            builder.format(move |f, record| match record.level() {
                log::Level::Error => writeln!(
                    f,
                    "{}: {}",
                    palette.error.paint(record.level()),
                    record.args()
                ),
                log::Level::Warn => writeln!(
                    f,
                    "{}: {}",
                    palette.warn.paint(record.level()),
                    record.args()
                ),
                log::Level::Info => writeln!(f, "{}", record.args()),
                log::Level::Debug => writeln!(
                    f,
                    "{}: {}",
                    palette.debug.paint(record.level()),
                    record.args()
                ),
                log::Level::Trace => writeln!(
                    f,
                    "{}: {}",
                    palette.trace.paint(record.level()),
                    record.args()
                ),
            });
        }

        builder.init();
    }
}

#[derive(Copy, Clone, Debug)]
struct Palette {
    error: yansi::Style,
    warn: yansi::Style,
    debug: yansi::Style,
    trace: yansi::Style,
}

impl Palette {
    pub fn colored() -> Self {
        Self {
            error: yansi::Style::new(yansi::Color::Red).bold(),
            warn: yansi::Style::new(yansi::Color::Yellow),
            debug: yansi::Style::new(yansi::Color::Blue),
            trace: yansi::Style::new(yansi::Color::Cyan),
        }
    }

    pub fn plain() -> Self {
        Self {
            error: yansi::Style::default(),
            warn: yansi::Style::default(),
            debug: yansi::Style::default(),
            trace: yansi::Style::default(),
        }
    }
}
