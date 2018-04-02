use std::path;
use std::ffi;

#[cfg(feature = "sass")]
use sass_rs;

use error::*;
use super::files;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum SassOutputStyle {
    Nested,
    Expanded,
    Compact,
    Compressed,
}

impl Default for SassOutputStyle {
    fn default() -> Self {
        SassOutputStyle::Nested
    }
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields, default)]
pub struct SassBuilder {
    pub import_dir: Option<String>,
    pub style: SassOutputStyle,
}

impl SassBuilder {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn build(self) -> SassCompiler {
        let Self { import_dir, style } = self;
        SassCompiler { import_dir, style }
    }
}

#[derive(Debug, PartialEq)]
pub struct SassCompiler {
    import_dir: Option<String>,
    style: SassOutputStyle,
}

impl Default for SassCompiler {
    fn default() -> Self {
        SassBuilder::default().build()
    }
}

impl SassCompiler {
    pub fn compile_file<S: AsRef<path::Path>, D: AsRef<path::Path>, F: AsRef<path::Path>>(
        &self,
        source: S,
        dest: D,
        file_path: F,
    ) -> Result<()> {
        self.compile_sass_internal(source.as_ref(), dest.as_ref(), file_path.as_ref())
    }

    #[cfg(feature = "sass")]
    fn compile_sass_internal(
        &self,
        source: &path::Path,
        dest: &path::Path,
        file_path: &path::Path,
    ) -> Result<()> {
        let mut sass_opts = sass_rs::Options::default();
        sass_opts.include_paths = self.import_dir.iter().cloned().collect();
        sass_opts.output_style = match self.style {
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
    fn compile_sass_internal(
        &self,
        source: &path::Path,
        dest: &path::Path,
        file_path: &path::Path,
    ) -> Result<()> {
        let rel_src = file_path
            .strip_prefix(source)
            .expect("file was found under the root");
        let dest_file = dest.join(rel_src);
        files::copy_file(file_path, &dest_file)
    }
}

pub fn is_sass_file(file_path: &path::Path) -> bool {
    file_path.extension() == Some(ffi::OsStr::new("scss"))
        || file_path.extension() == Some(ffi::OsStr::new("sass"))
}
