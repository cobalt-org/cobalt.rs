use std::ffi::OsStr;
use std::path;

use anyhow::Context as _;
use log::debug;
use serde::{Deserialize, Serialize};

use super::sass;
use super::{files, Minify};

use crate::error::Result;

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields, default)]
pub struct AssetsBuilder {
    pub sass: sass::SassBuilder,
    pub source: path::PathBuf,
}

impl AssetsBuilder {
    pub fn from_config(config: cobalt_config::Assets, source: &path::Path) -> Self {
        Self {
            sass: sass::SassBuilder::from_config(config.sass, source),
            source: source.to_owned(),
        }
    }

    pub fn build(self) -> Result<Assets> {
        let AssetsBuilder { sass, source } = self;

        let sass = sass.build();

        let assets = Assets { sass, source };
        Ok(assets)
    }
}

#[derive(Debug)]
pub struct Assets {
    sass: sass::SassCompiler,
    source: path::PathBuf,
}

impl Assets {
    pub fn process(
        &self,
        path: &path::Path,
        dest_root: &path::Path,
        minify: &Minify,
    ) -> Result<()> {
        let rel_src = path
            .strip_prefix(&self.source)
            .expect("file was found under the root");
        let dest_path = dest_root.join(rel_src);
        if sass::is_sass_file(path) {
            self.sass
                .compile_file(&self.source, dest_root, path, minify)?;
        } else if path.extension() == Some(OsStr::new("js")) {
            copy_and_minify_js(path, &dest_path, minify.js)?;
        } else if path.extension() == Some(OsStr::new("css")) {
            copy_and_minify_css(path, &dest_path, minify.css)?;
        } else {
            files::copy_file(path, &dest_path)?;
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
                .with_context(|| anyhow::format_err!("Could not create {}", parent.display()))?;
        }

        debug!(
            "Copying and minifying `{}` to `{}`",
            src_file.display(),
            dest_file.display()
        );
        let content = std::fs::read_to_string(src_file)?;
        let minified = minify(&content).map_err(|e| {
            anyhow::format_err!(
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
                .with_context(|| anyhow::format_err!("Could not create {}", parent.display()))?;
        }

        debug!(
            "Copying and minifying `{}` to `{}`",
            src_file.display(),
            dest_file.display()
        );
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
