use std::default::Default;
use std::ffi::OsStr;
use std::fs;
use std::io::Read;
use std::path;

use liquid;
use serde_json;
use serde_yaml;
use toml;

use error::*;
use files;

const DATA_DIR: &'static str = "_data";

#[derive(Debug, PartialEq, Clone)]
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SiteBuilder {
    pub name: Option<String>,
    pub description: Option<String>,
    pub base_url: Option<String>,
    pub data: Option<liquid::Object>,
    #[serde(skip)]
    pub data_dir: &'static str,
}

impl Default for SiteBuilder {
    fn default() -> SiteBuilder {
        SiteBuilder {
            name: None,
            description: None,
            base_url: None,
            data: None,
            data_dir: DATA_DIR,
        }
    }
}

impl SiteBuilder {
    pub fn build(self, root: &path::Path) -> Result<Site> {
        let SiteBuilder {
            name,
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

        let mut attributes = liquid::Object::new();
        if let Some(ref name) = name {
            attributes.insert("name".to_owned(), liquid::Value::str(name));
        }
        if let Some(ref description) = description {
            attributes.insert("description".to_owned(), liquid::Value::str(description));
        }
        if let Some(ref base_url) = base_url {
            attributes.insert("base_url".to_owned(), liquid::Value::str(base_url));
        }
        let mut data = data.unwrap_or_default();
        insert_data_dir(&mut data, &root.join(data_dir))?;
        if !data.is_empty() {
            attributes.insert("data".to_owned(), liquid::Value::Object(data));
        }

        Ok(Site {
               name,
               description,
               base_url,
               attributes,
           })
    }
}

#[derive(Debug, PartialEq, Default, Clone)]
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Site {
    pub name: Option<String>,
    pub description: Option<String>,
    pub base_url: Option<String>,
    pub attributes: liquid::Object,
}

fn deep_insert(data_map: &mut liquid::Object,
               file_path: &path::Path,
               target_key: String,
               data: liquid::Value)
               -> Result<()> {
    // now find the nested map it is supposed to be in
    let target_map = if let Some(path) = file_path.parent() {
        let mut map = data_map;
        for part in path.iter() {
            let key = part.to_str().ok_or_else(|| {
                format!("The data from {:?} can't be loaded as it contains non utf-8 characters",
                        path)
            })?;
            let cur_map = map;
            map = cur_map
                .entry(String::from(key))
                .or_insert_with(|| liquid::Value::Object(liquid::Object::new()))
                .as_object_mut()
                .ok_or_else(|| {
                                format!("Aborting: Duplicate in data tree. Would overwrite {:?} ",
                                        path)
                            })?;
        }
        map
    } else {
        data_map
    };

    match target_map.insert(target_key, data) {
        None => Ok(()),
        _ => {
            Err(format!("The data from {:?} can't be loaded: the key already exists",
                        file_path)
                    .into())
        }
    }
}

fn load_data(data_path: &path::Path) -> Result<liquid::Value> {
    let ext = data_path.extension().unwrap_or_else(|| OsStr::new(""));

    let data: liquid::Value;

    if ext == OsStr::new("yml") || ext == OsStr::new("yaml") {
        let reader = fs::File::open(data_path)?;
        data = serde_yaml::from_reader(reader)?;
    } else if ext == OsStr::new("json") {
        let reader = fs::File::open(data_path)?;
        data = serde_json::from_reader(reader)?;
    } else if ext == OsStr::new("toml") {
        let mut reader = fs::File::open(data_path)?;
        let mut text = String::new();
        reader.read_to_string(&mut text)?;
        data = toml::from_str(&text)?;
    } else {
        bail!("Failed to load of data {:?}: unknown file type '{:?}'.\n\
              Supported data files extensions are: yml, yaml, json and toml.",
              data_path,
              ext);
    }

    Ok(data)
}

fn insert_data_dir(data: &mut liquid::Object, data_root: &path::Path) -> Result<()> {
    let data_files_builder = files::FilesBuilder::new(data_root)?;
    let data_files = data_files_builder.build()?;
    for df in data_files.files() {
        let file_stem = df.file_stem()
            .expect("Files will always return with a stem");
        let file_stem = String::from(file_stem.to_str().unwrap());

        let full_path = data_root.join(df.clone());

        let data_fragment = load_data(&full_path)?;

        deep_insert(data, &df, file_stem, data_fragment)?;
    }

    Ok(())
}