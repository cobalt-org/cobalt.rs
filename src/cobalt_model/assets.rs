use std::path;
use std::ffi;

use super::sass;
use super::files;

use error::*;

#[derive(Debug, PartialEq, Default)]
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields, default)]
pub struct AssetsBuilder {
    pub sass: sass::SassBuilder,
    #[serde(skip)]
    pub source: Option<path::PathBuf>,
    #[serde(skip)]
    pub ignore: Vec<String>,
    #[serde(skip)]
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

        let mut files = files::FilesBuilder::new(&source)?;
        for line in ignore {
            files.add_ignore(&line)?;
        }
        for ext in template_extensions {
            files.add_ignore(&format!("*.{}", ext))?;
        }
        let files = files.build()?;
        let assets = Assets {
            sass,
            files,
            source,
        };
        Ok(assets)
    }
}

#[derive(Debug)]
pub struct Assets {
    pub sass: sass::SassCompiler,
    pub files: files::Files,
    pub source: path::PathBuf,
}

impl Assets {
    pub fn source(&self) -> &path::Path {
        self.source.as_path()
    }

    pub fn files(&self) -> &files::Files {
        &self.files
    }

    pub fn populate<P: AsRef<path::Path>>(&self, dest: P) -> Result<()> {
        self.populate_path(dest.as_ref())
    }

    fn populate_path(&self, dest: &path::Path) -> Result<()> {
        for file_path in self.files() {
            if file_path.extension() == Some(ffi::OsStr::new("scss")) {
                self.sass.compile_file(&self.source, dest, file_path)?;
            } else {
                let rel_src = file_path
                    .strip_prefix(&self.source)
                    .expect("file was found under the root");
                files::copy_file(&file_path, dest.join(rel_src).as_path())?;
            }
        }

        Ok(())
    }
}
