use std::env;
use std::ffi;
use std::path;

use clap;
use cobalt;
use regex;
use serde_yaml;

use args;
use error::*;

pub fn migrate_command_args() -> clap::App<'static, 'static> {
    clap::SubCommand::with_name("migrate")
        .about("migrate the cobalt project at the source dir")
        .args(&args::get_config_args())
}

pub fn migrate_command(matches: &clap::ArgMatches) -> Result<()> {
    migrate_config(matches.value_of("config"))
        .chain_err(|| "Failed to migrate config")?;

    let config = args::get_config(matches)?;
    let config = config.build()?;

    migrate_includes(config)?;

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
    info!("Migrating {:?}", config_path);

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

fn migrate_includes_path(content: String) -> Result<String> {
    lazy_static!{
        static ref REPLACEMENTS_REF: Vec<(regex::Regex, &'static str)> = vec![
            (r#"\{%\s*include\s*['"]_layouts/(.*?)['"]\s*%}"#, r#"{% include "$1" %}"#),
            (r#"\{\{\s*include\s*['"]_layouts/(.*?)['"]\s*}}"#, r#"{% include "$1" %}"#),
        ].into_iter()
            .map(|(r, s)| (regex::Regex::new(r).unwrap(), s))
            .collect();
    }
    let content = REPLACEMENTS_REF
        .iter()
        .fold(content, |content, &(ref search, ref replace)| {
            search.replace_all(&content, *replace).into_owned()
        });
    Ok(content)
}

fn migrate_includes(config: cobalt::Config) -> Result<()> {
    let layouts_dir = config.source.join(config.layouts_dir);
    let includes_dir = config.source.join(config.includes_dir);
    info!("Migrating (potential) snippets to {:?}", includes_dir);

    let files = cobalt::files::FilesBuilder::new(&layouts_dir)?
        .ignore_hidden(false)?
        .build()?;
    for file in files.files() {
        let rel_src = file.strip_prefix(&layouts_dir)
            .expect("file was found under the root");
        let dest = includes_dir.join(rel_src);
        cobalt::files::copy_file(&file, &dest)?;
    }

    let template_extensions: Vec<&ffi::OsStr> = config
        .template_extensions
        .iter()
        .map(ffi::OsStr::new)
        .collect();

    // HACK: Assuming its safe to run this conversion on everything
    let files = cobalt::files::FilesBuilder::new(&config.source)?
        .ignore_hidden(false)?
        .build()?;
    for file in files.files().filter(|p| {
        template_extensions.contains(&p.extension().unwrap_or_else(|| ffi::OsStr::new("")))
    }) {
        let content = cobalt::files::read_file(&file)?;
        let content = migrate_includes_path(content)?;
        cobalt::files::write_document_file(content, file)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn migrade_includes_path_ok() {
        let fixture = r#"{{ include "_layouts/_head.liquid" }}"#.to_owned();
        let expected = r#"{% include "_head.liquid" %}"#.to_owned();
        let actual = migrate_includes_path(fixture).unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn migrade_includes_path_complex() {
        let fixture =
            r#"Hi {{ include "_layouts/head.liquid" }} my {{ include "_layouts/foot.liquid" }}" du"#
                .to_owned();
        let expected = r#"Hi {% include "head.liquid" %} my {% include "foot.liquid" %}" du"#
            .to_owned();
        let actual = migrate_includes_path(fixture).unwrap();
        assert_eq!(expected, actual);
    }
}
