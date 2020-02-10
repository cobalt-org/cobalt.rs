use std::ffi;
use std::path;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Source {
    pub fs_path: path::PathBuf,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Dest {
    pub fs_path: path::PathBuf,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum AssetType {
    Sass,
    Raw,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct AssetTag;

pub fn derive_component(
    source_root: &path::Path,
    dest_root: &path::Path,
    file_path: path::PathBuf,
    is_sass_enabled: bool,
) -> (Source, Dest, AssetType) {
    let rel_src = file_path
        .strip_prefix(source_root)
        .expect("file was found under the root");
    let mut dest_file_path = dest_root.join(rel_src);
    let mut type_ = AssetType::Raw;
    if is_sass_enabled && is_sass_file(rel_src) {
        dest_file_path.set_extension("css");
        type_ = AssetType::Sass;
    }

    let source = Source { fs_path: file_path };
    let dest = Dest {
        fs_path: dest_file_path,
    };

    (source, dest, type_)
}

fn is_sass_file(file_path: &path::Path) -> bool {
    let ext = file_path.extension();
    ext == Some(ffi::OsStr::new("scss")) || ext == Some(ffi::OsStr::new("sass"))
}
