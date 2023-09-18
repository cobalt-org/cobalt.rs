#![cfg_attr(docsrs, feature(doc_auto_cfg))]

mod assets;
mod collection;
mod config;
mod document;
mod frontmatter;
mod pagination;
mod site;

pub mod path;

pub use self::assets::*;
pub use self::collection::*;
pub use self::config::*;
pub use self::document::*;
pub use self::frontmatter::*;
pub use self::pagination::*;
pub use self::site::*;
pub use liquid_core::model::DateTime;
pub use path::RelPath;

pub type Status = status::Status;
pub type Result<T, E = Status> = std::result::Result<T, E>;
