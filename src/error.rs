use std::result;
use std::io;
use std::error;
use std::fmt;
use walkdir;
use liquid;

// type alias because we always want to deal with CobaltErrors
pub type Result<T> = result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    Io(io::Error),
    Liquid(liquid::Error),
    WalkDir(walkdir::Error),
    Other(String),
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Error {
        Error::Io(err)
    }
}

impl From<liquid::Error> for Error {
    fn from(err: liquid::Error) -> Error {
        Error::Liquid(err)
    }
}

impl From<walkdir::Error> for Error {
    fn from(err: walkdir::Error) -> Error {
        Error::WalkDir(err)
    }
}

impl From<String> for Error {
    fn from(err: String) -> Error {
        Error::Other(err)
    }
}

impl<'a> From<&'a str> for Error {
    fn from(err: &'a str) -> Error {
        Error::Other(err.to_owned())
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::Io(ref err) => write!(f, "IO error: {}", err),
            Error::Liquid(ref err) => write!(f, "Liquid error: {}", err),
            Error::WalkDir(ref err) => write!(f, "walkdir error: {}", err),
            Error::Other(ref err) => write!(f, "error: {}", err),
        }
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::Io(ref err) => err.description(),
            Error::Liquid(ref err) => err.description(),
            Error::WalkDir(ref err) => err.description(),
            Error::Other(ref err) => err,
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        match *self {
            Error::Io(ref err) => Some(err),
            Error::Liquid(ref err) => Some(err),
            Error::WalkDir(ref err) => Some(err),
            Error::Other(_) => None,
        }
    }
}
