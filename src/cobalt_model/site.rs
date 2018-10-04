use std::ffi::OsStr;
use std::fs;
use std::path;

use liquid;
use serde_json;
use serde_yaml;
use toml;

use error::*;

use super::files;

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields, default)]
pub struct SiteBuilder {
    pub title: Option<String>,
    pub description: Option<String>,
    pub base_url: Option<String>,
    pub data: Option<liquid::value::Object>,
    pub data_dir: Option<path::PathBuf>,
}

impl SiteBuilder {
    pub fn build(self) -> Result<liquid::value::Object> {
        let SiteBuilder {
            title,
            description,
            base_url,
            data,
            data_dir,
        } = self;

        let base_url = base_url.map(|mut l| {
            if l.ends_with('/') {
                l.pop();
            }
            l
        });

        let mut attributes = liquid::value::Object::new();
        if let Some(title) = title {
            attributes.insert("title".into(), liquid::value::Value::scalar(title));
        }
        if let Some(description) = description {
            attributes.insert("description".into(), liquid::value::Value::scalar(description));
        }
        if let Some(base_url) = base_url {
            attributes.insert("base_url".into(), liquid::value::Value::scalar(base_url));
        }
        let mut data = data.unwrap_or_default();
        if let Some(ref data_dir) = data_dir {
            insert_data_dir(&mut data, data_dir)?;
        }
        if !data.is_empty() {
            attributes.insert("data".into(), liquid::value::Value::Object(data));
        }

        Ok(attributes)
    }
}

fn deep_insert(
    data_map: &mut liquid::value::Object,
    file_path: &path::Path,
    target_key: String,
    data: liquid::value::Value,
) -> Result<()> {
    // now find the nested map it is supposed to be in
    let target_map = if let Some(path) = file_path.parent() {
        let mut map = data_map;
        for part in path.iter() {
            let key = part.to_str().ok_or_else(|| {
                format!(
                    "The data from {:?} can't be loaded as it contains non utf-8 characters",
                    path
                )
            })?;
            let cur_map = map;
            map = cur_map
                .entry(String::from(key).into())
                .or_insert_with(|| liquid::value::Value::Object(liquid::value::Object::new()))
                .as_object_mut()
                .ok_or_else(|| {
                    format!(
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
        _ => Err(format!(
            "The data from {:?} can't be loaded: the key already exists",
            file_path
        ).into()),
    }
}

fn load_data(data_path: &path::Path) -> Result<liquid::value::Value> {
    let ext = data_path.extension().unwrap_or_else(|| OsStr::new(""));

    let data: liquid::value::Value;

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
        bail!(
            "Failed to load of data {:?}: unknown file type '{:?}'.\n\
             Supported data files extensions are: yml, yaml, json and toml.",
            data_path,
            ext
        );
    }

    Ok(data)
}

fn insert_data_dir(data: &mut liquid::value::Object, data_root: &path::Path) -> Result<()> {
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
            .chain_err(|| format!("Loading data from {:?} failed", full_path))?;

        deep_insert(data, rel_path, file_stem, data_fragment)
            .chain_err(|| format!("Merging data into {:?} failed", rel_path))?;
    }

    Ok(())
}
