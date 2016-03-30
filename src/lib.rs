#![deny(warnings)]

extern crate liquid;
extern crate pulldown_cmark;
extern crate walkdir;
extern crate crossbeam;
extern crate chrono;
extern crate yaml_rust;
extern crate rss;

#[macro_use]
extern crate nickel;

#[macro_use]
extern crate log;

pub use cobalt::build;
pub use cobalt::serve;
pub use error::Error;
pub use config::Config;

// modules
mod cobalt;
mod config;
pub mod error;
mod document;
