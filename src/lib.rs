// Deny warnings, except in dev mode
#![deny(warnings)]
// #![deny(missing_docs)]
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
extern crate walkdir;
extern crate yaml_rust;

extern crate itertools;

#[cfg(all(feature="syntax-highlight", not(windows)))]
extern crate syntect;

#[macro_use]
extern crate log;

#[macro_use]
extern crate error_chain;

#[macro_use]
extern crate lazy_static;

pub use cobalt::build;
pub use error::Error;
pub use config::Config;
pub use new::{create_new_project, create_new_post, create_new_layout, create_new_page};

pub mod error;

mod cobalt;
mod config;
mod document;
mod new;
mod slug;
mod files;

#[cfg(feature="syntax-highlight")]
mod syntax_highlight;
