#![warn(warnings)]

pub use crate::cobalt::build;
pub use crate::cobalt::classify_path;
pub use crate::cobalt_model::Config;
pub use crate::error::Error;

pub mod cobalt_model;
pub mod error;

mod cobalt;
mod document;

mod pagination;
mod syntax_highlight;

pub use crate::syntax_highlight::{list_syntax_themes, list_syntaxes};
