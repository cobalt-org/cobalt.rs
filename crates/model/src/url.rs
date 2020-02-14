use std::path;

use chrono::{Datelike, Timelike};
use liquid;

use crate::Result;

pub struct Url {
    pub url: String,
}

pub fn derive_page_url(front: &crate::page::Frontmatter, rel_path: &path::Path) -> Result<Url> {
    let perma_attributes = page_attributes(front, rel_path);
    let url = explode_permalink(front.permalink.as_str(), &perma_attributes)?;
    Ok(Url { url })
}

pub fn derive_dest(dest_root: &path::Path, url: &Url) -> crate::fs::Dest {
    let rel_path = format_url_as_file(&url.url);
    let fs_path = dest_root.join(rel_path);
    crate::fs::Dest { fs_path }
}

static PERMALINK_PARSER: once_cell::sync::Lazy<liquid::Parser> =
    once_cell::sync::Lazy::new(|| liquid::Parser::new());

pub fn explode_permalink(permalink: &str, attributes: &liquid::value::Object) -> Result<String> {
    let p = PERMALINK_PARSER.parse(permalink).map_err(|e| {
        crate::Status::new("Failed to parse permalink")
            .with_source(e)
            .context_with(|c| c.insert("permalink", permalink.to_owned()))
    })?;
    let mut p = p
        .render(attributes)
        .map_err(|e| crate::Status::new("Failed to render permalink").with_source(e))?;

    // Handle the user doing windows-style
    p = p.replace("\\", "/");

    // Handle cases where substitutions were blank
    p = p.replace("//", "/");

    if p.starts_with('/') {
        p.remove(0);
    }

    Ok(p)
}

pub fn page_attributes(
    front: &crate::page::Frontmatter,
    rel_path: &path::Path,
) -> liquid::value::Object {
    let mut attributes = liquid::value::Object::new();

    attributes.insert(
        "parent".into(),
        liquid::value::Value::scalar(format_path_variable(rel_path)),
    );

    let filename = rel_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_owned();
    attributes.insert("name".into(), liquid::value::Value::scalar(filename));

    attributes.insert("ext".into(), liquid::value::Value::scalar(".html"));

    // TODO(epage): Add `collection` (the collection's slug), see #257
    // or `parent.slug`, see #323

    attributes.insert(
        "slug".into(),
        liquid::value::Value::scalar(front.slug.clone()),
    );

    attributes.insert(
        "categories".into(),
        liquid::value::Value::scalar(itertools::join(
            front.categories.iter().map(cobalt_config::path::slugify),
            "/",
        )),
    );

    if let Some(ref date) = front.published_date {
        attributes.insert(
            "year".into(),
            liquid::value::Value::scalar(date.year().to_string()),
        );
        attributes.insert(
            "month".into(),
            liquid::value::Value::scalar(format!("{:02}", &date.month())),
        );
        attributes.insert(
            "i_month".into(),
            liquid::value::Value::scalar(date.month().to_string()),
        );
        attributes.insert(
            "day".into(),
            liquid::value::Value::scalar(format!("{:02}", &date.day())),
        );
        attributes.insert(
            "i_day".into(),
            liquid::value::Value::scalar(date.day().to_string()),
        );
        attributes.insert(
            "hour".into(),
            liquid::value::Value::scalar(format!("{:02}", &date.hour())),
        );
        attributes.insert(
            "minute".into(),
            liquid::value::Value::scalar(format!("{:02}", &date.minute())),
        );
        attributes.insert(
            "second".into(),
            liquid::value::Value::scalar(format!("{:02}", &date.second())),
        );
    }

    attributes.insert(
        "data".into(),
        liquid::value::Value::Object(front.data.clone()),
    );

    attributes
}

/// Convert the source file's relative path into a format useful for generating permalinks that
/// mirror the source directory hierarchy.
fn format_path_variable(source_file: &path::Path) -> String {
    let parent = source_file
        .parent()
        .and_then(|p| p.to_str())
        .unwrap_or("")
        .to_owned();
    let mut path = parent.replace("\\", "/");
    if path.starts_with("./") {
        path.remove(0);
    }
    if path.starts_with('/') {
        path.remove(0);
    }
    path
}

pub fn format_url_as_file(permalink: &str) -> path::PathBuf {
    let mut path = path::Path::new(&permalink);

    // remove the root prefix (leading slash on unix systems)
    if path.has_root() {
        let mut components = path.components();
        components.next();
        path = components.as_path();
    }

    let mut path_buf = path.to_path_buf();

    // explode the url if no extension was specified
    if path_buf.extension().is_none() {
        path_buf.push("index.html")
    }

    path_buf
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn explode_permalink_relative() {
        let attributes = liquid::value::Object::new();
        let actual = explode_permalink("relative/path", &attributes).unwrap();
        assert_eq!(actual, "relative/path");
    }

    #[test]
    fn explode_permalink_absolute() {
        let attributes = liquid::value::Object::new();
        let actual = explode_permalink("/abs/path", &attributes).unwrap();
        assert_eq!(actual, "abs/path");
    }

    #[test]
    fn explode_permalink_blank_substitution() {
        let attributes = liquid::value::Object::new();
        let actual = explode_permalink("//path/middle//end", &attributes).unwrap();
        assert_eq!(actual, "path/middle/end");
    }

    #[test]
    fn format_url_as_file_absolute() {
        let actual = format_url_as_file("/hello/world.html");
        assert_eq!(actual, path::Path::new("hello/world.html"));
    }

    #[test]
    fn format_url_as_file_no_explode() {
        let actual = format_url_as_file("/hello/world.custom");
        assert_eq!(actual, path::Path::new("hello/world.custom"));
    }

    #[test]
    fn format_url_as_file_explode() {
        let actual = format_url_as_file("/hello/world");
        assert_eq!(actual, path::Path::new("hello/world/index.html"));
    }

    #[test]
    fn format_path_variable_file() {
        let input = path::Path::new("/hello/world/file.liquid");
        let actual = format_path_variable(input);
        assert_eq!(actual, "hello/world");
    }

    #[test]
    fn format_path_variable_relative() {
        let input = path::Path::new("hello/world/file.liquid");
        let actual = format_path_variable(input);
        assert_eq!(actual, "hello/world");

        let input = path::Path::new("./hello/world/file.liquid");
        let actual = format_path_variable(input);
        assert_eq!(actual, "hello/world");
    }
}
