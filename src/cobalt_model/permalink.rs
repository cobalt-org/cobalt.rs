use liquid;
use std::path::{Path, PathBuf};

use error::*;

pub fn explode_permalink<S: AsRef<str>>(
  permalink: S,
  attributes: &liquid::Object,
) -> Result<String> {
  explode_permalink_string(permalink.as_ref(), attributes)
}

fn explode_permalink_string(permalink: &str, attributes: &liquid::Object) -> Result<String> {
  lazy_static! {
    static ref PERMALINK_PARSER: liquid::Parser = liquid::Parser::new();
  }
  let p = PERMALINK_PARSER.parse(permalink)?;
  let mut p = p.render(attributes)?;

  // Handle the user doing windows-style
  p = p.replace("\\", "/");

  // Handle cases where substutions were blank
  p = p.replace("//", "/");

  if p.starts_with('/') {
    p.remove(0);
  }

  Ok(p)
}

pub fn format_url_as_file<S: AsRef<str>>(permalink: S) -> PathBuf {
  format_url_as_file_str(permalink.as_ref())
}

fn format_url_as_file_str(permalink: &str) -> PathBuf {
  trace!("format_url_as_file_str: {}", permalink);
  let mut path = Path::new(&permalink);
  trace!("format_url_as_file_str: path {:?}", path);
  // remove the root prefix (leading slash on unix systems)
  if path.has_root() {
    let mut components = path.components();
    components.next();
    path = components.as_path();
  }

trace!("format_url_as_file_str: path after has_root {:?}", path);
  let mut path_buf = path.to_path_buf();

trace!("format_url_as_file_str: path_buf {:?}", path_buf);
  // explode the url if no extension was specified
  if path_buf.extension().is_none() {
    path_buf.push("index.html")
  }

trace!("format_url_as_file_str: path_buf after extension {:?}", path_buf);
  path_buf
}

#[cfg(test)]
mod test {
  use super::*;
  
  #[test]
  fn explode_permalink_relative() {
    let attributes = liquid::Object::new();
    let actual = explode_permalink("relative/path", &attributes).unwrap();
    assert_eq!(actual, "relative/path");
  }

  #[test]
  fn explode_permalink_absolute() {
    let attributes = liquid::Object::new();
    let actual = explode_permalink("/abs/path", &attributes).unwrap();
    assert_eq!(actual, "abs/path");
  }

  #[test]
  fn explode_permalink_blank_substitution() {
    let attributes = liquid::Object::new();
    let actual = explode_permalink("//path/middle//end", &attributes).unwrap();
    assert_eq!(actual, "path/middle/end");
  }

  #[test]
  fn format_url_as_file_absolute() {
    let actual = format_url_as_file("/hello/world.html");
    assert_eq!(actual, Path::new("hello/world.html"));
  }

  #[test]
  fn format_url_as_file_no_explode() {
    let actual = format_url_as_file("/hello/world.custom");
    assert_eq!(actual, Path::new("hello/world.custom"));
  }

  #[test]
  fn format_url_as_file_explode() {
    let actual = format_url_as_file("/hello/world");
    assert_eq!(actual, Path::new("hello/world/index.html"));
  }
}
