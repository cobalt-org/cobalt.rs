#![warn(warnings)]

extern crate chrono;
extern crate ignore;
extern crate itertools;
extern crate jsonfeed;
extern crate liquid;
extern crate normalize_line_endings;
extern crate pulldown_cmark;
extern crate regex;
extern crate rss;
extern crate serde_json;
extern crate serde_yaml;
extern crate toml;
extern crate unidecode;
extern crate walkdir;

#[cfg(feature = "sass")]
extern crate sass_rs;

#[cfg(all(feature = "syntax-highlight"))]
extern crate syntect;

#[macro_use]
extern crate log;

#[macro_use]
extern crate error_chain;

#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate serde;

#[cfg(test)]
#[macro_use]
extern crate difference;

pub use cobalt::build;
pub use cobalt_model::Config;
pub use cobalt_model::ConfigBuilder;
pub use error::Error;

pub mod cobalt_model;
pub mod error;

mod cobalt;
mod document;

mod syntax_highlight;

pub use syntax_highlight::{list_syntax_themes, list_syntaxes};
