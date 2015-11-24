#![deny(warnings)]

extern crate liquid;
extern crate markdown;
extern crate walkdir;
extern crate crossbeam;

pub use cobalt::build;
pub use error::Error;

// modules
mod cobalt;
mod error;
mod document;
