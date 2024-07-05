//! Markdown Fenced Code Block Highlighting

#![cfg_attr(docsrs, feature(doc_auto_cfg))]
#![warn(clippy::print_stderr)]
#![warn(clippy::print_stdout)]

#[cfg(feature = "syntax")]
mod syntax;
#[cfg(feature = "syntax")]
pub use syntax::*;

pub use raw::*;
mod raw;
