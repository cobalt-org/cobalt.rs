use std::ffi::OsStr;
use std::fs;
use std::path;

use cobalt_config::DateTime;
use failure::ResultExt;
use liquid;
use log::debug;
use serde::{Deserialize, Serialize};
use serde_json;
use serde_yaml;
use toml;

use super::files;

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields, default)]
pub struct Site {
    pub title: Option<kstring::KString>,
    pub description: Option<kstring::KString>,
    pub base_url: Option<kstring::KString>,
    pub sitemap: Option<cobalt_config::RelPath>,
    pub data: Option<liquid::Object>,
    pub data_dir: &'static str,
    /// The time at which the `cobalt` binary built the site
    pub time: DateTime,
}

impl Site {
    pub fn from_config(config: cobalt_config::Site) -> Self {
        let cobalt_config::Site {
            title,
            description,
            base_url,
            sitemap,
            data,
            data_dir,
        } = config;

        let base_url = base_url.map(|mut l| {
            if l.ends_with('/') {
                let mut other = String::from(l.as_str());
                other.pop();
                l = kstring::KString::from_string(other);
            }
            l
        });

        Self {
            title,
            description,
            base_url,
            sitemap,
            data,
            data_dir,
            time: DateTime::now(),
        }
    }

    pub fn load(&self, source: &std::path::Path) -> Result<liquid::Object> {
        let mut attributes = liquid::Object::new();
        if let Some(title) = self.title.as_ref() {
            attributes.insert(
                "title".into(),
                liquid::model::Value::scalar(kstring::KString::from_ref(title)),
            );
        }
        if let Some(description) = self.description.as_ref() {
            attributes.insert(
                "description".into(),
                liquid::model::Value::scalar(kstring::KString::from_ref(description)),
            );
        }
        if let Some(base_url) = self.base_url.as_ref() {
            attributes.insert(
                "base_url".into(),
                liquid::model::Value::scalar(kstring::KString::from_ref(base_url)),
            );
        }
        attributes.insert("time".into(), liquid::model::Value::scalar(self.time));

        let mut data = self.data.clone().unwrap_or_default();
        let data_path = source.join(&self.data_dir);
        insert_data_dir(&mut data, &data_path)?;
        if !data.is_empty() {
            attributes.insert("data".into(), liquid::model::Value::Object(data));
        }

        Ok(attributes)
    }
}

fn deep_insert(
    data_map: &mut liquid::Object,
    file_path: &path::Path,
    target_key: String,
    data: liquid::model::Value,
) -> Result<()> {
    // now find the nested map it is supposed to be in
    let target_map = if let Some(path) = file_path.parent() {
        let mut map = data_map;
        for part in path.iter() {
            let key = part.to_str().ok_or_else(|| {
                failure::format_err!(
                    "The data from {:?} can't be loaded as it contains non utf-8 characters",
                    path
                )
            })?;
            let cur_map = map;
            let key = kstring::KString::from_ref(key);
            map = cur_map
                .entry(key)
                .or_insert_with(|| liquid::model::Value::Object(liquid::Object::new()))
                .as_object_mut()
                .ok_or_else(|| {
                    failure::format_err!(
                        "Aborting: Duplicate in data tree. Would overwrite {:?} ",
                        path
                    )
                })?;
        }
        map
    } else {
        data_map
    };

    match target_map.insert(target_key.into(), data) {
        None => Ok(()),
        _ => Err(failure::format_err!(
            "The data from {:?} can't be loaded: the key already exists",
            file_path
        )),
    }
}

fn load_data(data_path: &path::Path) -> Result<liquid::model::Value> {
    let ext = data_path.extension().unwrap_or_else(|| OsStr::new(""));

    let data: liquid::model::Value;

    if ext == OsStr::new("yml") || ext == OsStr::new("yaml") {
        let reader = fs::File::open(data_path)?;
        data = serde_yaml::from_reader(reader)?;
    } else if ext == OsStr::new("json") {
        let reader = fs::File::open(data_path)?;
        data = serde_json::from_reader(reader)?;
    } else if ext == OsStr::new("toml") {
        let text = files::read_file(data_path)?;
        data = toml::from_str(&text)?;
    } else {
        failure::bail!(
            "Failed to load of data {:?}: unknown file type '{:?}'.\n\
             Supported data files extensions are: yml, yaml, json and toml.",
            data_path,
            ext
        );
    }

    Ok(data)
}

fn insert_data_dir(data: &mut liquid::Object, data_root: &path::Path) -> Result<()> {
    debug!("Loading data from {:?}", data_root);

    let data_files_builder = files::FilesBuilder::new(data_root)?;
    let data_files = data_files_builder.build()?;
    for full_path in data_files.files() {
        let rel_path = full_path
            .strip_prefix(data_root)
            .expect("file was found under the root");

        let file_stem = full_path
            .file_stem()
            .expect("Files will always return with a stem");
        let file_stem = String::from(file_stem.to_str().unwrap());
        let data_fragment = load_data(&full_path)
            .with_context(|_| format!("Loading data from {:?} failed", full_path))?;

        deep_insert(data, rel_path, file_stem, data_fragment)
            .with_context(|_| format!("Merging data into {:?} failed", rel_path))?;
    }

    Ok(())
}
