use std::default::Default;
use std::path::Path;
use std::fs::File;
use std::io::Read;
use glob::Pattern;
use error::Result;
use yaml_rust::YamlLoader;

#[derive(Debug, PartialEq)]
pub struct Config {
    pub source: String,
    pub dest: String,
    pub layouts: String,
    pub posts: String,
    pub sass: String,
    pub template_extensions: Vec<String>,
    pub rss: Option<String>,
    pub name: Option<String>,
    pub description: Option<String>,
    pub link: Option<String>,
    pub ignore: Vec<Pattern>,
}

impl Default for Config {
    fn default() -> Config {
        Config {
            source: "./".to_owned(),
            dest: "./".to_owned(),
            layouts: "_layouts".to_owned(),
            posts: "_posts".to_owned(),
            sass: "_sass".to_owned(),
            template_extensions: vec!["md".to_owned(), "liquid".to_owned()],
            rss: None,
            name: None,
            description: None,
            link: None,
            ignore: vec![],
        }
    }
}

impl Config {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Config> {
        let mut buffer = String::new();
        let mut f = try!(File::open(path));
        try!(f.read_to_string(&mut buffer));

        let yaml = try!(YamlLoader::load_from_str(&buffer));
        let yaml = match yaml.get(0) {
            Some(ref y) => y.clone(),
            None => return Ok(Default::default()),
        };

        let mut config = Config {
            name: yaml["name"].as_str().map(|s| s.to_owned()),
            rss: yaml["rss"].as_str().map(|s| s.to_owned()),
            description: yaml["description"].as_str().map(|s| s.to_owned()),
            ..Default::default()
        };

        if let Some(source) = yaml["source"].as_str() {
            config.source = source.to_owned();
        };

        if let Some(dest) = yaml["dest"].as_str() {
            config.dest = dest.to_owned();
        };

        if let Some(layouts) = yaml["layouts"].as_str() {
            config.layouts = layouts.to_owned();
        };

        if let Some(posts) = yaml["posts"].as_str() {
            config.posts = posts.to_owned();
        };

        if let Some(extensions) = yaml["template_extensions"].as_vec() {
            config.template_extensions = extensions.iter()
                                                   .filter_map(|k| {
                                                       k.as_str().map(|k| k.to_owned())
                                                   })
                                                   .collect();
        };

        if let Some(link) = yaml["link"].as_str() {
            let mut link = link.to_owned();
            if !link.ends_with("/") {
                link = link + "/";
            }
            config.link = Some(link);
        };

        if let Some(patterns) = yaml["ignore"].as_vec() {
            config.ignore = patterns.iter()
                                    .filter_map(|k| k.as_str())
                                    .filter_map(|k| Pattern::new(k).ok())
                                    .collect();
        };

        Ok(config)
    }
}

#[test]
fn test_from_file_ok() {
    let result = Config::from_file("tests/fixtures/config/.cobalt.yml");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(),
               Config {
                   dest: "./dest".to_owned(),
                   layouts: "_my_layouts".to_owned(),
                   posts: "_my_posts".to_owned(),
                   ..Default::default()
               });
}

#[test]
fn test_from_file_rss() {
    let result = Config::from_file("tests/fixtures/config/rss.yml");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(),
               Config {
                   rss: Some("rss.xml".to_owned()),
                   name: Some("My blog!".to_owned()),
                   description: Some("Blog description".to_owned()),
                   link: Some("http://example.com/".to_owned()),
                   ..Default::default()
               });
}

#[test]
fn test_from_file_empty() {
    let result = Config::from_file("tests/fixtures/config/empty.yml");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Config { ..Default::default() });
}

#[test]
fn test_from_file_invalid_syntax() {
    let result = Config::from_file("tests/fixtures/config/invalid_syntax.yml");
    assert!(result.is_err());
}

#[test]
fn test_from_file_not_found() {
    let result = Config::from_file("tests/fixtures/config/config_does_not_exist.yml");
    assert!(result.is_err());
}
