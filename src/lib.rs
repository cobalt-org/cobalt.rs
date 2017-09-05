// Deny warnings, except in dev mode
#![deny(warnings)]
#![allow(unknown_lints)]
#![allow(unused_doc_comment)] // error-chain 0.11 should fix this.
#![cfg_attr(feature="dev", warn(warnings))]

// Stuff we want clippy to ignore
#![cfg_attr(feature="cargo-clippy", allow(
        cyclomatic_complexity,
        too_many_arguments,
        ))]

extern crate chrono;
extern crate ignore;
extern crate liquid;
extern crate pulldown_cmark;
extern crate regex;
extern crate rss;
extern crate jsonfeed;
extern crate walkdir;
extern crate serde_yaml;

extern crate itertools;

#[cfg(all(feature="syntax-highlight", not(windows)))]
extern crate syntect;

#[macro_use]
extern crate log;

#[macro_use]
extern crate clap;

#[macro_use]
extern crate error_chain;

#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate serde;

pub use cobalt::build;
pub use error::Error;
pub use config::Config;
pub use config::Dump;
pub use new::{create_new_project, create_new_document};

pub mod error;

mod cobalt;
mod config;
mod document;
mod new;
mod slug;
mod files;
mod datetime;
mod frontmatter;

mod legacy;
mod syntax_highlight;

pub use syntax_highlight::{list_syntax_themes, list_syntaxes};
