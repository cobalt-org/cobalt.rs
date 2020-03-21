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
pub use liquid_core::model::scalar::DateTime;

type Status = status::Status;
type Result<T, E = Status> = std::result::Result<T, E>;
