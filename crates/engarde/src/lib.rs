//! Markdown Fenced Code Block Highlighting

#![cfg_attr(docsrs, feature(doc_auto_cfg))]

#[cfg(feature = "syntax")]
mod syntax;
#[cfg(feature = "syntax")]
pub use syntax::*;

pub use raw::*;
mod raw;
