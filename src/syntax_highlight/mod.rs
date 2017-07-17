#[cfg(all(feature="syntax-highlight", not(windows)))]
mod syntect;
#[cfg(all(feature="syntax-highlight", not(windows)))]
pub use self::syntect::*;

#[cfg(any(not(feature="syntax-highlight"), windows))]
mod null;
#[cfg(any(not(feature="syntax-highlight"), windows))]
pub use self::null::*;
