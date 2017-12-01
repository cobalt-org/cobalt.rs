use std::env;
use std::path;

use clap;
use cobalt;
use serde_yaml;

use args;
use error::*;

pub fn migrate_command_args() -> clap::App<'static, 'static> {
    clap::SubCommand::with_name("migrate")
        .about("migrate the cobalt project at the source dir")
        .args(&args::get_config_args())
}

pub fn migrate_command(matches: &clap::ArgMatches) -> Result<()> {
    info!("Migrating");

    migrate_config(matches.value_of("config"))
        .chain_err(|| "Failed to migrate config")?;

    let config = args::get_config(matches)?;
    let _config = config.build()?;

    Ok(())
}

fn migrate_config(config_path: Option<&str>) -> Result<()> {
    let config_path = if let Some(config_path) = config_path {
        path::Path::new(config_path).to_path_buf()
    } else {
        let cwd = env::current_dir().expect("How does this fail?");
        let config_path = cobalt::cobalt_model::files::find_project_file(&cwd, ".cobalt.yml")
            .unwrap_or_else(|| cwd.join(".cobalt.yml"));
        config_path
    };

    let content = cobalt::cobalt_model::files::read_file(&config_path);
    let config = if let Ok(content) = content {
        let config: cobalt::legacy_model::GlobalConfig = serde_yaml::from_str(&content)?;
        config
    } else {
        cobalt::legacy_model::GlobalConfig::default()
    };
    let config: cobalt::ConfigBuilder = config.into();
    let content = config.to_string();
    cobalt::cobalt_model::files::write_document_file(content, config_path)?;

    Ok(())
}
