use std::ffi;
use std::path;

use serde::{Deserialize, Serialize};

use super::files;
use crate::cobalt_model::Minify;
use crate::error::Result;
pub(crate) use cobalt_config::SassOutputStyle;

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields, default)]
pub struct SassBuilder {
    pub import_dir: path::PathBuf,
    pub style: SassOutputStyle,
}

impl SassBuilder {
    pub fn from_config(config: cobalt_config::Sass, source: &path::Path) -> Self {
        Self {
            style: config.style,
            import_dir: source.join(config.import_dir),
        }
    }

    pub fn build(self) -> SassCompiler {
        let Self { import_dir, style } = self;
        SassCompiler { import_dir, style }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct SassCompiler {
    import_dir: path::PathBuf,
    style: SassOutputStyle,
}

impl SassCompiler {
    pub fn compile_file<S: AsRef<path::Path>, D: AsRef<path::Path>, F: AsRef<path::Path>>(
        &self,
        source: S,
        dest: D,
        file_path: F,
        minify: &Minify,
    ) -> Result<()> {
        self.compile_sass_internal(source.as_ref(), dest.as_ref(), file_path.as_ref(), minify)
    }

    #[cfg(feature = "sass")]
    fn compile_sass_internal(
        &self,
        source: &path::Path,
        dest: &path::Path,
        file_path: &path::Path,
        minify: &Minify,
    ) -> Result<()> {
        let sass_opts = grass::Options::default()
            .style(match self.style {
                SassOutputStyle::Nested | SassOutputStyle::Expanded => grass::OutputStyle::Expanded,
                SassOutputStyle::Compact | SassOutputStyle::Compressed => {
                    grass::OutputStyle::Compressed
                }
            })
            .load_path(&self.import_dir);
        let content = if let Some(file_path) = file_path.to_str() {
            grass::from_path(file_path, &sass_opts)?
        } else {
            let raw = std::fs::read_to_string(file_path)?;
            grass::from_string(raw, &sass_opts)?
        };

        let rel_src = file_path
            .strip_prefix(source)
            .expect("file was found under the root");
        let mut dest_file = dest.join(rel_src);
        dest_file.set_extension("css");

        #[cfg(feature = "html-minifier")]
        let content = if minify.css {
            use html_minifier::css::minify;
            minify(&content)
                .map_err(|e| {
                    anyhow::format_err!(
                        "Could not minify saas file {} error {}",
                        source.to_string_lossy(),
                        e
                    )
                })?
                .to_string()
        } else {
            content
        };

        files::write_document_file(content, dest_file)
    }

    #[cfg(not(feature = "sass"))]
    fn compile_sass_internal(
        &self,
        source: &path::Path,
        dest: &path::Path,
        file_path: &path::Path,
        minify: &Minify,
    ) -> Result<()> {
        let rel_src = file_path
            .strip_prefix(source)
            .expect("file was found under the root");
        let dest_file = dest.join(rel_src);
        files::copy_file(file_path, &dest_file)
    }
}

impl Default for SassCompiler {
    fn default() -> Self {
        SassBuilder::default().build()
    }
}

pub(crate) fn is_sass_file(file_path: &path::Path) -> bool {
    file_path.extension() == Some(ffi::OsStr::new("scss"))
        || file_path.extension() == Some(ffi::OsStr::new("sass"))
}
