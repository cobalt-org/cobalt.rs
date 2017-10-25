use std::default::Default;

use error::*;

const DATA_DIR: &'static str = "_data";

#[derive(Debug, PartialEq)]
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SiteBuilder {
    pub name: Option<String>,
    pub description: Option<String>,
    pub base_url: Option<String>,
    #[serde(skip)]
    pub data_dir: &'static str,
}

impl Default for SiteBuilder {
    fn default() -> SiteBuilder {
        SiteBuilder {
            name: None,
            description: None,
            base_url: None,
            data_dir: DATA_DIR,
        }
    }
}

impl SiteBuilder {
    pub fn build(self) -> Result<SiteBuilder> {
        let SiteBuilder {
            name,
            description,
            base_url,
            data_dir,
        } = self;
        let base_url = base_url.map(|mut l| {
                                        if l.ends_with('/') {
                                            l.pop();
                                        }
                                        l
                                    });
        Ok(SiteBuilder {
               name,
               description,
               base_url,
               data_dir,
           })
    }
}
