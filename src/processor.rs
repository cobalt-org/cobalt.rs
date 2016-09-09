use walkdir::DirEntry;
use config::Config;
use error::Result;

// Include 
pub use processors::*;

pub trait Processor {
    fn match_dir(entry: &DirEntry, config: &Config) -> bool;

    fn process(entry: DirEntry, config: &Config) -> Result<()>;
}
