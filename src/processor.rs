use walkdir::DirEntry;
use config::Config;
use error::Result;

// Include 
pub use processors::*;

pub trait Processor {
    fn match_dir(&mut self, entry: &DirEntry, config: &Config) -> bool;

    fn process(&mut self, entry: DirEntry, config: &Config) -> Result<()>;
}
