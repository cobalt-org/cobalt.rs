#[cfg(feature = "syntax-highlight")]
mod syntect;
#[cfg(feature = "syntax-highlight")]
pub use self::syntect::*;

#[cfg(not(feature = "syntax-highlight"))]
mod null;
#[cfg(not(feature = "syntax-highlight"))]
pub use self::null::*;
