use std::fmt;
use std::fmt::Display;

pub use failure::Error;
pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
struct SourceWrapper {
    message: String,
}

impl Display for SourceWrapper {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Display::fmt(&self.message, f)
    }
}

impl std::error::Error for SourceWrapper {}

#[derive(Debug)]
struct StatusWrapper {
    status: cobalt_config::Status,
    cause: SourceWrapper,
}

impl Display for StatusWrapper {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Display::fmt(&self.status, f)
    }
}

impl failure::Fail for StatusWrapper {
    fn cause(&self) -> Option<&dyn failure::Fail> {
        Some(&self.cause)
    }
}

pub trait HasSource<T> {
    fn map_err_with_sources(self) -> Result<T>;
}

impl<T> HasSource<T> for cobalt_config::Result<T> {
    fn map_err_with_sources(self) -> Result<T> {
        self.map_err(|status| {
            let cause: Option<SourceWrapper> =
                status.sources().next().map(|source| SourceWrapper {
                    message: format!("{}", source),
                });

            if let Some(cause) = cause {
                return failure::Error::from(StatusWrapper { status, cause });
            }

            failure::Error::from(status)
        })
    }
}
