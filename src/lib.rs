// Deny warnings, except in dev mode
#![deny(warnings)]
// #![deny(missing_docs)]
#![cfg_attr(feature="dev", warn(warnings))]

// Stuff we want clippy to ignore
#![cfg_attr(feature="cargo-clippy", allow(
        cyclomatic_complexity,
        too_many_arguments,
        ))]

extern crate liquid;
extern crate pulldown_cmark;
extern crate walkdir;
extern crate chrono;
extern crate yaml_rust;
extern crate rss;
extern crate glob;
extern crate regex;

extern crate itertools;

#[cfg(all(feature="syntax-highlight", not(windows)))]
extern crate syntect;

#[macro_use]
extern crate log;

#[macro_use]
extern crate error_chain;

pub use cobalt::build;
pub use error::Error;
pub use config::Config;
pub use new::create_new_project;

// modules
mod cobalt;
mod config;
pub mod error;
mod document;
mod new;
mod slug;

#[cfg(feature="syntax-highlight")]
mod syntax_highlight;

#[macro_use]
extern crate lazy_static;
