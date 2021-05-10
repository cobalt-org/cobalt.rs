use std::path;

use super::sass;
use super::{files, Minify};

use crate::error::*;
use failure::ResultExt;
use std::ffi::OsStr;

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

    pub fn populate<P: AsRef<path::Path>>(&self, dest: P, minify: &Minify) -> Result<()> {
        self.populate_path(dest.as_ref(), minify)
    }

    fn populate_path(&self, dest: &path::Path, minify: &Minify) -> Result<()> {
        for file_path in self.files() {
            let rel_src = file_path
                .strip_prefix(self.source())
                .expect("file was found under the root");
            let dest_path = dest.join(rel_src);
            if sass::is_sass_file(file_path.as_path()) {
                self.sass
                    .compile_file(self.source(), dest, file_path.as_path(), minify)?;
            } else if file_path.extension() == Some(OsStr::new("js")) {
                copy_and_minify_js(file_path.as_path(), dest_path.as_path(), minify.js)?;
            } else if file_path.extension() == Some(OsStr::new("css")) {
                copy_and_minify_css(file_path.as_path(), dest_path.as_path(), minify.css)?;
            } else {
                files::copy_file(&file_path, dest_path.as_path())?;
            }
        }

        Ok(())
    }
}

#[cfg(feature = "html-minifier")]
fn copy_and_minify_css(src_file: &path::Path, dest_file: &path::Path, minify: bool) -> Result<()> {
    if minify {
        use html_minifier::css::minify;
        // create target directories if any exist
        if let Some(parent) = dest_file.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|_| failure::format_err!("Could not create {}", parent.display()))?;
        }

        debug!("Copying and minifying {:?} to {:?}", src_file, dest_file);
        let content = std::fs::read_to_string(src_file)?;
        let minified = minify(&content).map_err(|e| {
            failure::format_err!(
                "Could not minify css file {} error {}",
                src_file.to_string_lossy(),
                e
            )
        })?;
        std::fs::write(dest_file, minified)?;
    } else {
        files::copy_file(src_file, dest_file)?;
    }
    Ok(())
}

#[cfg(feature = "html-minifier")]
fn copy_and_minify_js(src_file: &path::Path, dest_file: &path::Path, minify: bool) -> Result<()> {
    if minify {
        use html_minifier::js::minify;
        // create target directories if any exist
        if let Some(parent) = dest_file.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|_| failure::format_err!("Could not create {}", parent.display()))?;
        }

        debug!("Copying and minifying {:?} to {:?}", src_file, dest_file);
        let content = std::fs::read_to_string(src_file)?;
        std::fs::write(dest_file, minify(&content))?;
    } else {
        files::copy_file(src_file, dest_file)?;
    }
    Ok(())
}

#[cfg(not(feature = "html-minifier"))]
fn copy_and_minify_css(src_file: &path::Path, dest_file: &path::Path, _minify: bool) -> Result<()> {
    files::copy_file(src_file, dest_file)?;
    Ok(())
}

#[cfg(not(feature = "html-minifier"))]
fn copy_and_minify_js(src_file: &path::Path, dest_file: &path::Path, _minify: bool) -> Result<()> {
    files::copy_file(src_file, dest_file)?;
    Ok(())
}
