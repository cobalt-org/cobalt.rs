use std::env;
use std::io::Write;
use std::path;

use failure::ResultExt;

use crate::error::*;

#[derive(Clone, Debug, PartialEq, Eq, clap::Args)]
pub struct ConfigArgs {
    /// Config file to use [default: _cobalt.yml]
    #[clap(short, long, value_name = "FILE", parse(from_os_str))]
    config: Option<path::PathBuf>,

    /// Site destination folder [default: ./]
    #[clap(short, long, value_name = "DIR", parse(from_os_str))]
    destination: Option<path::PathBuf>,

    /// Include drafts.
    #[clap(long)]
    drafts: bool,

    /// Ignore drafts.
    #[clap(long, conflicts_with = "drafts")]
    no_drafts: bool,
}

impl ConfigArgs {
    pub fn load_config(&self) -> Result<cobalt_config::Config> {
        let config_path = self.config.as_deref();

        // Fetch config information if available
        let mut config = if let Some(config_path) = config_path {
            cobalt_config::Config::from_file(config_path).with_context(|_| {
                failure::format_err!("Error reading config file {:?}", config_path)
            })?
        } else {
            let cwd = env::current_dir().expect("How does this fail?");
            cobalt_config::Config::from_cwd(cwd)?
        };

        config.abs_dest = self
            .destination
            .as_deref()
            .map(|d| {
                std::fs::create_dir_all(d)?;
                d.canonicalize()
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

pub fn get_logging(level: log::Level) -> Result<env_logger::Builder> {
    let mut builder = env_logger::Builder::new();

    builder.filter(None, level.to_level_filter());

    if level == log::LevelFilter::Trace {
        builder.format_timestamp_secs();
    } else {
        builder.format(|f, record| {
            writeln!(
                f,
                "[{}] {}",
                record.level().to_string().to_lowercase(),
                record.args()
            )
        });
    }

    Ok(builder)
}
