#![cfg_attr(docsrs, feature(doc_auto_cfg))]
#![warn(clippy::print_stderr)]
#![warn(clippy::print_stdout)]

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

pub use crate::syntax_highlight::SyntaxHighlight;
