use std::path;

use cobalt_config::SassOutputStyle;
#[cfg(feature = "sass")]
use sass_rs;

#[cfg(feature = "sass")]
use super::files;
use crate::error::*;

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields, default)]
pub struct SassBuilder {
    pub import_dir: Option<String>,
    pub style: SassOutputStyle,
}

impl SassBuilder {
    pub fn from_config(config: cobalt_config::Sass, source: &path::Path) -> Self {
        Self {
            style: config.style,
            import_dir: source
                .join(config.import_dir)
                .into_os_string()
                .into_string()
                .ok(),
        }
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
    #[cfg(feature = "sass")]
    pub fn compile_file(&self, source: &path::Path, dest: &path::Path) -> Result<()> {
        let mut sass_opts = sass_rs::Options::default();
        sass_opts.include_paths = self.import_dir.iter().cloned().collect();
        sass_opts.output_style = match self.style {
            SassOutputStyle::Nested => sass_rs::OutputStyle::Nested,
            SassOutputStyle::Expanded => sass_rs::OutputStyle::Expanded,
            SassOutputStyle::Compact => sass_rs::OutputStyle::Compact,
            SassOutputStyle::Compressed => sass_rs::OutputStyle::Compressed,
        };
        let content = sass_rs::compile_file(source, sass_opts).map_err(failure::err_msg)?;

        files::write_document_file(content, dest)
    }

    #[cfg(not(feature = "sass"))]
    pub fn compile_file(&self, _source: &path::Path, _dest: &path::Path) -> Result<()> {
        failure::bail!("Cannot compile sass files");
    }
}
