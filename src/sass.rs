use std::path;

#[cfg(feature = "sass")]
use sass_rs;

use error::*;
use files;

#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone)]
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum SassOutputStyle {
    Nested,
    Expanded,
    Compact,
    Compressed,
}

const SASS_IMPORT_DIR: &'static str = "_sass";

#[derive(Debug, PartialEq)]
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SassOptions {
    #[serde(skip)]
    pub import_dir: &'static str,
    pub style: SassOutputStyle,
}

impl Default for SassOptions {
    fn default() -> SassOptions {
        SassOptions {
            import_dir: SASS_IMPORT_DIR,
            style: SassOutputStyle::Nested,
        }
    }
}

pub fn compile_sass<S: AsRef<path::Path>, D: AsRef<path::Path>, F: AsRef<path::Path>>
    (config: &SassOptions,
     source: S,
     dest: D,
     file_path: F)
     -> Result<()> {
    compile_sass_internal(config, source.as_ref(), dest.as_ref(), file_path.as_ref())
}

#[cfg(feature = "sass")]
fn compile_sass_internal(config: &SassOptions,
                         source: &path::Path,
                         dest: &path::Path,
                         file_path: &path::Path)
                         -> Result<()> {
    let mut sass_opts = sass_rs::Options::default();
    sass_opts.include_paths = vec![source
                                       .join(&config.import_dir)
                                       .into_os_string()
                                       .into_string()
                                       .unwrap()];
    sass_opts.output_style = match config.style {
        SassOutputStyle::Nested => sass_rs::OutputStyle::Nested,
        SassOutputStyle::Expanded => sass_rs::OutputStyle::Expanded,
        SassOutputStyle::Compact => sass_rs::OutputStyle::Compact,
        SassOutputStyle::Compressed => sass_rs::OutputStyle::Compressed,
    };
    let content = sass_rs::compile_file(file_path, sass_opts)?;

    let rel_src = file_path
        .strip_prefix(source)
        .expect("file was found under the root");
    let mut dest_file = dest.join(rel_src);
    dest_file.set_extension("css");

    files::write_document_file(content, dest_file)
}

#[cfg(not(feature = "sass"))]
fn compile_sass_internal(_config: &SassOptions,
                         source: &path::Path,
                         dest: &path::Path,
                         file_path: &path::Path)
                         -> Result<()> {
    let src_file = source.join(file_path);
    files::copy_file(src_file.as_path(), dest.join(file_path).as_path())
}
