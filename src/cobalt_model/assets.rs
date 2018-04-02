use std::path;

use super::sass;
use super::files;

use error::*;

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields, default)]
pub struct AssetsBuilder {
    pub sass: sass::SassBuilder,
    pub source: Option<path::PathBuf>,
    pub ignore: Vec<String>,
    pub template_extensions: Vec<String>,
}

impl AssetsBuilder {
    pub fn build(self) -> Result<Assets> {
        let AssetsBuilder {
            sass,
            source,
            ignore,
            template_extensions,
        } = self;

        let sass = sass.build();

        let source = source.ok_or_else(|| "No asset source provided")?;

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
