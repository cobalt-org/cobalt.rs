use std::path;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Source {
    pub fs_path: path::PathBuf,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Dest {
    pub fs_path: path::PathBuf,
}
