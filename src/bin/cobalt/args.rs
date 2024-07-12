use std::io::Write;
use std::path;

use anyhow::Context as _;

use crate::error::Result;

#[derive(Clone, Debug, PartialEq, Eq, clap::Args)]
pub(crate) struct ConfigArgs {
    /// Config file to use [default: _cobalt.yml]
    #[arg(short, long, value_name = "FILE")]
    config: Option<path::PathBuf>,

    /// Include drafts.
    #[arg(long)]
    drafts: bool,

    /// Ignore drafts.
    #[arg(long, conflicts_with = "drafts")]
    no_drafts: bool,
}

impl ConfigArgs {
    pub(crate) fn load_config(&self) -> Result<cobalt_config::Config> {
        let config_path = self.config.as_deref();

        // Fetch config information if available
        let mut config = if let Some(config_path) = config_path {
            cobalt_config::Config::from_file(config_path).with_context(|| {
                anyhow::format_err!("Error reading config file {}", config_path.display())
            })?
        } else {
            cobalt_config::Config::from_cwd(".")?
        };

        if let Some(drafts) = self.drafts() {
            config.include_drafts = drafts;
        }

        Ok(config)
    }

    pub(crate) fn drafts(&self) -> Option<bool> {
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

pub(crate) fn init_logging(
    level: clap_verbosity_flag::Verbosity<clap_verbosity_flag::InfoLevel>,
    colored: bool,
) {
    if let Some(level) = level.log_level() {
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
            builder.format(move |f, record| {
                let level = record.level();
                let args = record.args();
                match record.level() {
                    log::Level::Error => writeln!(f, "{ERROR}{level}{ERROR:#}: {args}"),
                    log::Level::Warn => writeln!(f, "{WARN}{level}{WARN:#}: {args}"),
                    log::Level::Info => writeln!(f, "{args}"),
                    log::Level::Debug => writeln!(f, "{DEBUG}{level}{DEBUG:#}: {args}"),
                    log::Level::Trace => writeln!(f, "{TRACE}{level}{TRACE:#}: {args}"),
                }
            });
        }

        builder.init();
    }
}

const ERROR: anstyle::Style = anstyle::AnsiColor::Red.on_default().bold();
const WARN: anstyle::Style = anstyle::AnsiColor::Yellow.on_default();
const DEBUG: anstyle::Style = anstyle::AnsiColor::Blue.on_default();
const TRACE: anstyle::Style = anstyle::AnsiColor::Cyan.on_default();
