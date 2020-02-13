use std::path;

use legion::query::IntoQuery;

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

    pub fn populate(&self, dest: &path::Path, world: &mut legion::world::World) -> Result<()> {
        #[cfg(feature = "sass")]
        let is_sass_enabled = true;
        #[cfg(not(feature = "sass"))]
        let is_sass_enabled = false;

        world.insert(
            (cobalt_model::assets::AssetTag,),
            self.files().files().map(|file_path| {
                let (dest, type_) = cobalt_model::assets::derive_component(
                    self.files.root(),
                    dest,
                    &file_path,
                    is_sass_enabled,
                );
                let source = cobalt_model::fs::Source { fs_path: file_path };
                (source, dest, type_)
            }),
        );

        Ok(())
    }

    pub fn process(&self, world: &legion::world::World) -> Result<()> {
        let query = <(
            legion::query::Read<cobalt_model::fs::Source>,
            legion::query::Read<cobalt_model::fs::Dest>,
            legion::query::Read<cobalt_model::assets::AssetType>,
        )>::query()
        .filter(legion::prelude::tag::<cobalt_model::assets::AssetTag>());
        for (source, dest, type_) in query.iter_immutable(world) {
            match *type_ {
                cobalt_model::assets::AssetType::Sass => {
                    self.sass.compile_file(&source.fs_path, &dest.fs_path)?
                }
                cobalt_model::assets::AssetType::Raw => {
                    files::copy_file(&source.fs_path, &dest.fs_path)?
                }
            }
        }

        Ok(())
    }
}
