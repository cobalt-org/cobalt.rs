use std::env;
use std::ffi;
use std::path;

use clap;
use cobalt;
use cobalt::cobalt_model;
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

    migrate_content(&config)?;
    migrate_front(&config)?;

    Ok(())
}

fn migrate_config(config_path: Option<&str>) -> Result<()> {
    let config_path = if let Some(config_path) = config_path {
        path::Path::new(config_path).to_path_buf()
    } else {
        let cwd = env::current_dir().expect("How does this fail?");
        cobalt_model::files::find_project_file(&cwd, ".cobalt.yml")
            .unwrap_or_else(|| cwd.join(".cobalt.yml"))
    };
    info!("Migrating {:?}", config_path);

    let content = cobalt_model::files::read_file(&config_path);
    let config = if let Ok(content) = content {
        let config: cobalt::legacy_model::GlobalConfig = serde_yaml::from_str(&content)?;
        config
    } else {
        cobalt::legacy_model::GlobalConfig::default()
    };
    let config: cobalt::ConfigBuilder = config.into();
    let content = config.to_string();
    cobalt_model::files::write_document_file(content, config_path)?;

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
        .fold(content, |content, &(ref search, replace)| {
            search.replace_all(&content, replace).into_owned()
        });
    Ok(content)
}

fn migrate_variables(content: String) -> Result<String> {
    lazy_static!{
        static ref REPLACEMENTS_REF: Vec<(regex::Regex, &'static str)> = vec![
            // From tests
            (r#"\{\{\s*title\s*}}"#, "{{ page.title }}"),
            (r#"\{\{\s*(.*?)\.path\s*}}"#, "{{ $1.permalink }}"),
            (r#"\{\{\s*path\s*}}"#, "{{ page.permalink }}"),
            (r#"\{\{\s*content\s*}}"#, "{{ page.content }}"),
            (r#"\{\{\s*previous\.(.*?)\s*}}"#, "{{ page.previous.$1 }}"),
            (r#"\{\{\s*next\.(.*?)\s*}}"#, "{{ page.next.$1 }}"),
            (r#"\{%\s*if\s+is_post\s*%}"#, "{% if page.is_post %}"),
            (r#"\{%\s*if\s+previous\s*%}"#, "{% if page.previous %}"),
            (r#"\{%\s*if\s+next\s*%}"#, "{% if page.next %}"),
            (r#"\{%\s*if\s+draft\s*%}"#, "{% if page.is_draft %}"),
            // from johannhof.github.io
            (r#"\{\{\s*route\s*}}"#, "{{ page.data.route }}"),
            (r#"\{%\s*if\s+route\s*"#, "{% if page.data.route "),
            (r#"\{\{\s*date\s*"#, "{{ page.published_date "),
            // From blog
            (r#"\{\{\s*post\.date\s*"#, "{{ post.published_date "),
            // From booyaa.github.io
            (r#"\{%\s*assign\s+word_count\s*=\s*content"#, "{% assign word_count = page.content"),
            (r#"\{%\s*assign\s+year\s*=\s*post.path"#, "{% assign year = post.permalink"),
            (r#"\{%\s*assign\s+tags_list\s*=\s*post.tags"#, "{% assign tags_list = post.data.tags"),
            (r#"\{%\s*assign\s+tags\s*=\s*post.tags"#, "{% assign tags = post.data.tags"),
            // From deep-blog
            (r#"\{%\s*if\s+lang\s*%}"#, "{% if page.data.lang %}"),
            (r#"\{\{\s*lang\s*}}"#, "{{ page.data.lang }}"),
            (r#"\{%\s*if\s+comments\s*%}"#, "{% if page.data.comments %}"),
            (r#"\{%\s*if\s+dsq_thread_id\s*%}"#, "{% if page.data.dsq_thread_id %}"),
            (r#"\{\{\s*dsq_thread_id\s*}}"#, "{{ page.data.dsq_thread_id }}"),
            (r#"\{%\s*if\s+img_cover\s*%}"#, "{% if page.data.img_cover %}"),
            (r#"\{\{\s*img_cover\s*}}"#, "{{ page.data.img_cover }}"),
            (r#"\{%\s*if\s+post\.img_cover\s*%}"#, "{% if post.data.img_cover %}"),
            (r#"\{\{\s*post\.img_cover\s*}}"#, "{{ post.data.img_cover }}"),
            (r#"\{\{\s*post\.author\s*}}"#, "{{ post.data.author }}"),
            // fnordig.de
            (r#"\{%\s*assign\s+postyear\s*=\s*post.date"#,
             "{% assign postyear = post.published_date"),
            // hellorust
            (r#"\{\{\s*author\s*}}"#, "{{ page.data.author }}"),
            // mre
            (r#"\{\{\s*title"#, "{{ page.title"),
            (r#"\{%\s*if\s+title\s*!=\s*""\s*%}"#, r#"{% if page.title != "" %}"#),
            (r#"\{%-\s*if\s+title\s*!=\s*""\s*%}"#, r#"{%- if page.title != "" %}"#),
        ].into_iter()
            .map(|(r, s)| (regex::Regex::new(r).unwrap(), s))
            .collect();
    }
    let content = REPLACEMENTS_REF
        .iter()
        .fold(content, |content, &(ref search, replace)| {
            search.replace_all(&content, replace).into_owned()
        });
    Ok(content)
}

fn migrate_content(config: &cobalt::Config) -> Result<()> {
    let layouts_dir = config.source.join(config.layouts_dir);
    let includes_dir = config.source.join(config.includes_dir);
    info!("Migrating (potential) snippets to {:?}", includes_dir);

    let files = cobalt_model::files::FilesBuilder::new(&layouts_dir)?
        .ignore_hidden(false)?
        .build()?;
    for file in files.files() {
        let rel_src = file.strip_prefix(&layouts_dir)
            .expect("file was found under the root");
        let dest = includes_dir.join(rel_src);
        cobalt_model::files::copy_file(&file, &dest)?;
    }

    let template_extensions: Vec<&ffi::OsStr> = config
        .template_extensions
        .iter()
        .map(ffi::OsStr::new)
        .collect();

    // HACK: Assuming its safe to run this conversion on everything
    let files = cobalt_model::files::FilesBuilder::new(&config.source)?
        .ignore_hidden(false)?
        .build()?;
    for file in files.files().filter(|p| {
        template_extensions.contains(&p.extension().unwrap_or_else(|| ffi::OsStr::new("")))
    }) {
        let content = cobalt_model::files::read_file(&file)?;
        let content = migrate_includes_path(content)?;
        let content = migrate_variables(content)?;
        cobalt_model::files::write_document_file(content, file)?;
    }

    Ok(())
}

fn migrate_front_format(document_content: &str) -> Result<String> {
    let document = cobalt::legacy_model::DocumentBuilder::parse(document_content)?;
    let document: cobalt_model::DocumentBuilder<cobalt_model::FrontmatterBuilder> = document.into();
    Ok(document.to_string())
}

fn migrate_front(config: &cobalt::Config) -> Result<()> {
    info!("Migrating frontmatter");

    let template_extensions: Vec<&ffi::OsStr> = config
        .template_extensions
        .iter()
        .map(ffi::OsStr::new)
        .collect();

    let mut page_files = cobalt_model::files::FilesBuilder::new(&config.source)?;
    page_files
        .add_ignore(&format!("!{}", config.posts.dir))?
        .add_ignore(&format!("!{}/**", config.posts.dir))?
        .add_ignore(&format!("{}/**/_*", config.posts.dir))?
        .add_ignore(&format!("{}/**/_*/**", config.posts.dir))?;
    for line in &config.ignore {
        page_files.add_ignore(line.as_str())?;
    }
    let page_files = page_files.build()?;
    for file_path in page_files.files().filter(|p| {
        template_extensions.contains(&p.extension().unwrap_or_else(|| ffi::OsStr::new("")))
    }) {
        let content = cobalt_model::files::read_file(&file_path)?;
        let content = migrate_front_format(&content)?;
        cobalt_model::files::write_document_file(content, file_path)?;
    }

    if let Some(ref drafts_dir) = config.posts.drafts_dir {
        debug!("Draft directory: {:?}", drafts_dir);
        let drafts_root = config.source.join(&drafts_dir);
        let mut draft_files = cobalt_model::files::FilesBuilder::new(drafts_root.as_path())?;
        for line in &config.ignore {
            draft_files.add_ignore(line.as_str())?;
        }
        let draft_files = draft_files.build()?;
        for file_path in draft_files.files().filter(|p| {
            template_extensions.contains(&p.extension().unwrap_or_else(|| ffi::OsStr::new("")))
        }) {
            let content = cobalt_model::files::read_file(&file_path)?;
            let content = migrate_front_format(&content)?;
            cobalt_model::files::write_document_file(content, file_path)?;
        }
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

    #[test]
    fn migrate_variables_empty() {
        let fixture = r#""#.to_owned();
        let expected = r#""#.to_owned();
        let actual = migrate_variables(fixture).unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn migrate_variables_title() {
        let fixture = r#"<h1>{{ path }}</h1>"#.to_owned();
        let expected = r#"<h1>{{ page.permalink }}</h1>"#.to_owned();
        let actual = migrate_variables(fixture).unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn migrate_variables_path() {
        let fixture = r#"<h2>{{ title }}</h2>"#.to_owned();
        let expected = r#"<h2>{{ page.title }}</h2>"#.to_owned();
        let actual = migrate_variables(fixture).unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn migrate_variables_content() {
        let fixture = r#"<h2>{{ content }}</h2>"#.to_owned();
        let expected = r#"<h2>{{ page.content }}</h2>"#.to_owned();
        let actual = migrate_variables(fixture).unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn migrate_variables_scoped() {
        let fixture = r#"<a href="{{post.path}}">{{ post.title }}</a>"#.to_owned();
        let expected = r#"<a href="{{ post.permalink }}">{{ post.title }}</a>"#.to_owned();
        let actual = migrate_variables(fixture).unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn migrate_variables_previous() {
        let fixture = r#"<a ref="/{{previous.path}}">&laquo; {{previous.title}}</a>"#.to_owned();
        let expected =
            r#"<a ref="/{{ page.previous.permalink }}">&laquo; {{ page.previous.title }}</a>"#
                .to_owned();
        let actual = migrate_variables(fixture).unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn migrate_variables_next() {
        let fixture = r#"<a class="next" href="/{{next.path}}">&laquo; {{next.title}}</a>"#
            .to_owned();
        let expected =
            r#"<a class="next" href="/{{ page.next.permalink }}">&laquo; {{ page.next.title }}</a>"#
                .to_owned();
        let actual = migrate_variables(fixture).unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn migrate_variables_if_is_post() {
        let fixture = r#"{% if is_post %}"#.to_owned();
        let expected = r#"{% if page.is_post %}"#.to_owned();
        let actual = migrate_variables(fixture).unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn migrate_variables_if_previous() {
        let fixture = r#"{% if previous %}"#.to_owned();
        let expected = r#"{% if page.previous %}"#.to_owned();
        let actual = migrate_variables(fixture).unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn migrate_variables_if_next() {
        let fixture = r#"{% if next %}"#.to_owned();
        let expected = r#"{% if page.next %}"#.to_owned();
        let actual = migrate_variables(fixture).unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn migrate_variables_if_is_draft() {
        let fixture = r#"{% if draft %}"#.to_owned();
        let expected = r#"{% if page.is_draft %}"#.to_owned();
        let actual = migrate_variables(fixture).unwrap();
        assert_eq!(expected, actual);
    }
}
