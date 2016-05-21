#![deny(warnings)]

extern crate liquid;
extern crate pulldown_cmark;
extern crate walkdir;
extern crate crossbeam;
extern crate chrono;
extern crate yaml_rust;
extern crate rss;
extern crate glob;
extern crate regex;

#[macro_use]
extern crate log;

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
