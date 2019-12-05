use std::path;

use super::files;
use super::sass;

use crate::error::*;

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields, default)]
pub struct AssetsBuilder {
    pub sass: sass::SassBuilder,
    pub source: Option<path::PathBuf>,
    pub ignore: Vec<String>,
    pub template_extensions: Vec<String>,
}

impl AssetsBuilder {
    pub fn from_config(
        config: cobalt_config::Assets,
        source: &path::Path,
        ignore: &[String],
        template_extensions: &[String],
    ) -> Self {
        Self {
            sass: sass::SassBuilder::from_config(config.sass, source),
            source: Some(source.to_owned()),
            ignore: ignore.to_vec(),
            template_extensions: template_extensions.to_vec(),
        }
    }

    pub fn build(self) -> Result<Assets> {
        let AssetsBuilder {
            sass,
            source,
            ignore,
            template_extensions,
        } = self;

        let sass = sass.build();

        let source = source.ok_or_else(|| failure::err_msg("No asset source provided"))?;

        let mut files = files::FilesBuilder::new(source)?;
        for line in ignore {
            files.add_ignore(&line)?;
        }
        for ext in template_extensions {
            files.add_ignore(&format!("*.{}", ext))?;
        }
        let files = files.build()?;
        let assets = Assets { sass, files };
        Ok(assets)
    }
}

#[derive(Debug)]
pub struct Assets {
    sass: sass::SassCompiler,
    files: files::Files,
}

impl Assets {
    pub fn source(&self) -> &path::Path {
        self.files.root()
    }

    pub fn files(&self) -> &files::Files {
        &self.files
    }

    pub fn populate<P: AsRef<path::Path>>(&self, dest: P) -> Result<()> {
        self.populate_path(dest.as_ref())
    }

    fn populate_path(&self, dest: &path::Path) -> Result<()> {
        for file_path in self.files() {
            if sass::is_sass_file(file_path.as_path()) {
                self.sass.compile_file(self.source(), dest, file_path)?;
            } else {
                let rel_src = file_path
                    .strip_prefix(self.source())
                    .expect("file was found under the root");
                files::copy_file(&file_path, dest.join(rel_src).as_path())?;
            }
        }

        Ok(())
    }
}
