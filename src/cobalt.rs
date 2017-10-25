use std::fs::{self, File};
use std::collections::HashMap;
use std::io::Write;
use std::io::Read;
use std::path::Path;
use std::ffi::OsStr;
use liquid::Value;
use rss;
use jsonfeed::Feed;
use jsonfeed;
use serde_yaml;
use serde_json;
use toml;

#[cfg(feature = "sass")]
use sass_rs;

use datetime;
use document::Document;
use error::*;
use config::{Config, SortOrder};
#[cfg(feature = "sass")]
use config::SassOutputStyle;
use files::FilesBuilder;
use frontmatter;

fn deep_insert(data_map: &mut HashMap<String, Value>,
               file_path: &Path,
               target_key: String,
               data: Value)
               -> Result<()> {
    // now find the nested map it is supposed to be in
    let target_map = if let Some(path) = file_path.parent() {
        let mut map = data_map;
        for part in path.iter() {
            if let Some(key) = part.to_str() {
                let cur_map = map;
                map = if let Some(sub_map) = cur_map
                       .entry(String::from(key))
                       .or_insert_with(|| Value::Object(HashMap::new()))
                       .as_object_mut() {
                    sub_map
                } else {
                    bail!("Aborting: Dublicate in data tree. Would overwrite {} ",
                          &key)
                }
            } else {
                bail!("The data from {:?} can't be loaded as it contains non utf-8 characters",
                      path);
            }
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

/// The primary build function that transforms a directory into a site
pub fn build(config: &Config) -> Result<()> {
    trace!("Build configuration: {:?}", config);

    let source = config.source.as_path();
    let dest = config.destination.as_path();

    let template_extensions: Vec<&OsStr> =
        config.template_extensions.iter().map(OsStr::new).collect();

    let layouts = source.join(&config.layouts);
    let mut layouts_cache = HashMap::new();
    let posts_path = source.join(&config.posts);

    debug!("Layouts directory: {:?}", layouts);
    debug!("Posts directory: {:?}", posts_path);
    debug!("Draft mode enabled: {}", config.include_drafts);
    if config.include_drafts {
        debug!("Draft directory: {:?}", config.drafts);
    }

    let mut documents = vec![];

    let ignore_dest = {
        let ignore_dest = dest.join("**/*");
        let ignore_dest = ignore_dest
            .to_str()
            .ok_or_else(|| format!("Cannot convert pathname {:?} to UTF-8", dest))?
            .to_owned();
        Some(ignore_dest)
    };

    let mut page_files = FilesBuilder::new(source)?;
    page_files
        .add_ignore(&format!("!{}", config.posts))?
        .add_ignore(&format!("!{}/**", config.posts))?
        .add_ignore(&format!("{}/**/_*", config.posts))?
        .add_ignore(&format!("{}/**/_*/**", config.posts))?;
    for line in &config.ignore {
        page_files.add_ignore(line.as_str())?;
    }
    if let Some(ref ignore_dest) = ignore_dest {
        page_files.add_ignore(ignore_dest)?;
    }
    let page_files = page_files.build()?;
    for file_path in page_files.files().filter(|p| {
        template_extensions.contains(&p.extension().unwrap_or_else(|| OsStr::new("")))
    }) {
        // if the document is in the posts folder it's considered a post
        let src_path = source.join(file_path.as_path());
        let is_post = src_path.starts_with(posts_path.as_path());

        let mut default_front = frontmatter::FrontmatterBuilder::new()
            .set_post(is_post)
            .set_draft(false)
            .set_excerpt_separator(config.excerpt_separator.clone());
        if is_post {
            default_front = default_front.set_permalink(config.post_path.clone());
        }

        let doc = Document::parse(source, &file_path, &file_path, default_front)
            .chain_err(|| format!("Failed to parse {:?}", src_path))?;
        if !doc.front.is_draft || config.include_drafts {
            documents.push(doc);
        }
    }

    if config.include_drafts {
        let drafts_root = source.join(&config.drafts);
        let mut draft_files = FilesBuilder::new(drafts_root.as_path())?;
        for line in &config.ignore {
            draft_files.add_ignore(line.as_str())?;
        }
        let draft_files = draft_files.build()?;
        for file_path in draft_files.files().filter(|p| {
            template_extensions.contains(&p.extension().unwrap_or_else(|| OsStr::new("")))
        }) {
            let new_path = posts_path.join(&file_path);
            let new_path = new_path
                .strip_prefix(source)
                .expect("Entry not in source folder");

            let is_post = true;

            let mut default_front = frontmatter::FrontmatterBuilder::new()
                .set_post(is_post)
                .set_draft(true)
                .set_excerpt_separator(config.excerpt_separator.clone());
            if is_post {
                default_front = default_front.set_permalink(config.post_path.clone());
            }

            let doc = Document::parse(&drafts_root, &file_path, new_path, default_front)
                .chain_err(|| {
                               let src_path = drafts_root.join(file_path);
                               format!("Failed to parse {:?}", src_path)
                           })?;
            documents.push(doc);
        }
    }

    // load data files
    let mut data_map: HashMap<String, Value> = HashMap::new();
    let data_root = source.join(&config.site.data_dir);
    let data_files_builder = FilesBuilder::new(data_root.as_path())?;
    let data_files = data_files_builder.build()?;

    for df in data_files.files() {

        let ext = df.extension().unwrap_or_else(|| OsStr::new(""));
        let file_stem = df.file_stem()
            .expect("Files will always return with a stem");

        let file_name = String::from(file_stem.to_str().unwrap());
        let full_path = data_root.join(df.clone());
        let data: Value;

        if ext == OsStr::new("yml") || ext == OsStr::new("yaml") {
            let reader = File::open(full_path)?;
            data = serde_yaml::from_reader(reader)?;
        } else if ext == OsStr::new("json") {
            let reader = File::open(full_path)?;
            data = serde_json::from_reader(reader)?;
        } else if ext == OsStr::new("toml") {
            let mut reader = File::open(full_path)?;
            let mut text = String::new();
            reader.read_to_string(&mut text)?;
            data = toml::from_str(&text)?;
        } else {
            warn!("Skipping loading of data {:?}: unknown file type.",
                  full_path);
            warn!("Supported data files extensions are: yml, yaml, json and toml.");
            continue;
        }

        deep_insert(&mut data_map, &df, file_name, data)?;
    }

    // now wrap it all into the global site-object
    let mut site = HashMap::new();
    site.insert("data".to_owned(), Value::Object(data_map));

    // January 1, 1970 0:00:00 UTC, the beginning of time
    let default_date = datetime::DateTime::default();

    let (mut posts, documents): (Vec<Document>, Vec<Document>) =
        documents.into_iter().partition(|x| x.front.is_post);

    // sort documents by date, if there's no date (none was provided or it couldn't be read) then
    // fall back to the default date
    posts.sort_by(|a, b| {
                      b.front
                          .published_date
                          .unwrap_or(default_date)
                          .cmp(&a.front.published_date.unwrap_or(default_date))
                  });

    match config.post_order {
        SortOrder::Asc => posts.reverse(),
        SortOrder::Desc => (),
    }

    // collect all posts attributes to pass them to other posts for rendering
    let simple_posts_data: Vec<Value> = posts
        .iter()
        .map(|x| Value::Object(x.attributes.clone()))
        .collect();

    trace!("Generating posts");
    for (i, post) in &mut posts.iter_mut().enumerate() {
        trace!("Generating {}", post.url_path);

        // posts are in reverse date order, so previous post is the next in the list (+1)
        if let Some(previous) = simple_posts_data.get(i + 1) {
            post.attributes
                .insert("previous".to_owned(), previous.clone());
        }
        if i >= 1 {
            if let Some(next) = simple_posts_data.get(i - 1) {
                post.attributes.insert("next".to_owned(), next.clone());
            }
        }

        for dump in config.dump.iter().filter(|d| d.is_doc()) {
            trace!("Dumping {:?}", dump);
            let (content, ext) = post.render_dump(*dump)?;
            let mut file_path = post.file_path.clone();
            let file_name = file_path
                .file_stem()
                .and_then(|p| p.to_str())
                .expect("page must have file name")
                .to_owned();
            let file_name = format!("_{}.{}.{}", file_name, dump, ext);
            file_path.set_file_name(file_name);
            trace!("Generating {:?}", file_path);
            create_document_file(content, dest.join(file_path))?;
        }

        let mut context = post.get_render_context(&simple_posts_data);
        context.set_val("site", Value::Object(site.clone()));

        post.render_excerpt(&mut context, source, &config.syntax_highlight.theme)
            .chain_err(|| format!("Failed to render excerpt for {:?}", post.file_path))?;
        let post_html = post.render(&mut context,
                                    source,
                                    &layouts,
                                    &mut layouts_cache,
                                    &config.syntax_highlight.theme)
            .chain_err(|| format!("Failed to render for {:?}", post.file_path))?;
        create_document_file(post_html, dest.join(&post.file_path))?;
    }

    // check if we should create an RSS file and create it!
    if let Some(ref path) = config.rss {
        create_rss(path, dest, config, &posts)?;
    }
    // check if we should create an jsonfeed file and create it!
    if let Some(ref path) = config.jsonfeed {
        create_jsonfeed(path, dest, config, &posts)?;
    }

    // during post rendering additional attributes such as content were
    // added to posts. collect them so that non-post documents can access them
    let posts_data: Vec<Value> = posts
        .into_iter()
        .map(|x| Value::Object(x.attributes))
        .collect();

    trace!("Generating other documents");
    for mut doc in documents {
        trace!("Generating {}", doc.url_path);

        for dump in config.dump.iter().filter(|d| d.is_doc()) {
            trace!("Dumping {:?}", dump);
            let (content, ext) = doc.render_dump(*dump)?;
            let mut file_path = doc.file_path.clone();
            let file_name = file_path
                .file_stem()
                .and_then(|p| p.to_str())
                .expect("page must have file name")
                .to_owned();
            let file_name = format!("_{}.{}.{}", file_name, dump, ext);
            file_path.set_file_name(file_name);
            trace!("Generating {:?}", file_path);
            create_document_file(content, dest.join(file_path))?;
        }

        let mut context = doc.get_render_context(&posts_data);
        context.set_val("site", Value::Object(site.clone()));

        let doc_html = doc.render(&mut context,
                                  source,
                                  &layouts,
                                  &mut layouts_cache,
                                  &config.syntax_highlight.theme)
            .chain_err(|| format!("Failed to render for {:?}", doc.file_path))?;
        create_document_file(doc_html, dest.join(doc.file_path))?;
    }

    // copy all remaining files in the source to the destination
    // compile SASS along the way
    {
        info!("Copying remaining assets");

        let mut asset_files = FilesBuilder::new(source)?;
        for line in &config.ignore {
            asset_files.add_ignore(line.as_str())?;
        }
        if let Some(ref ignore_dest) = ignore_dest {
            asset_files.add_ignore(ignore_dest)?;
        }
        let asset_files = asset_files.build()?;
        for file_path in asset_files.files().filter(|p| {
            !template_extensions.contains(&p.extension().unwrap_or_else(|| OsStr::new("")))
        }) {
            if file_path.extension() == Some(OsStr::new("scss")) {
                compile_sass(config, source, dest, file_path)?;
            } else {
                let src_file = source.join(&file_path);
                copy_file(src_file.as_path(), dest.join(file_path).as_path())?;
            }
        }
    }

    Ok(())
}

// creates a new RSS file with the contents of the site blog
fn create_rss(path: &str, dest: &Path, config: &Config, posts: &[Document]) -> Result<()> {
    let name = config
        .site
        .name
        .as_ref()
        .ok_or(ErrorKind::ConfigFileMissingFields)?;
    let description = config
        .site
        .description
        .as_ref()
        .ok_or(ErrorKind::ConfigFileMissingFields)?;
    let link = config
        .site
        .base_url
        .as_ref()
        .ok_or(ErrorKind::ConfigFileMissingFields)?;

    let items: Result<Vec<rss::Item>> = posts.iter().map(|doc| doc.to_rss(link)).collect();
    let items = items?;

    let channel = rss::ChannelBuilder::default()
        .title(name.to_owned())
        .link(link.to_owned())
        .description(description.to_owned())
        .items(items)
        .build()?;

    let rss_string = channel.to_string();
    trace!("RSS data: {}", rss_string);

    let rss_path = dest.join(path);

    let mut rss_file = File::create(&rss_path)?;
    rss_file
        .write_all(br#"<?xml version="1.0" encoding="UTF-8"?>"#)?;
    rss_file.write_all(&rss_string.into_bytes())?;
    rss_file.write_all(b"\n")?;

    info!("Created RSS file at {}", rss_path.display());
    Ok(())
}
// creates a new jsonfeed file with the contents of the site blog
fn create_jsonfeed(path: &str, dest: &Path, config: &Config, posts: &[Document]) -> Result<()> {
    let name = config
        .site
        .name
        .as_ref()
        .ok_or(ErrorKind::ConfigFileMissingFields)?;
    let description = config
        .site
        .description
        .as_ref()
        .ok_or(ErrorKind::ConfigFileMissingFields)?;
    let link = config
        .site
        .base_url
        .as_ref()
        .ok_or(ErrorKind::ConfigFileMissingFields)?;

    let jsonitems = posts.iter().map(|doc| doc.to_jsonfeed(link)).collect();

    let feed = Feed {
        title: name.to_string(),
        items: jsonitems,
        home_page_url: Some(link.to_string()),
        description: Some(description.to_string()),
        ..Default::default()
    };

    let jsonfeed_string = jsonfeed::to_string(&feed).unwrap();
    let jsonfeed_path = dest.join(path);
    let mut jsonfeed_file = File::create(&jsonfeed_path)?;
    jsonfeed_file.write_all(&jsonfeed_string.into_bytes())?;

    info!("Created jsonfeed file at {}", jsonfeed_path.display());
    Ok(())
}

fn compile_sass<S: AsRef<Path>, D: AsRef<Path>, F: AsRef<Path>>(config: &Config,
                                                                source: S,
                                                                dest: D,
                                                                file_path: F)
                                                                -> Result<()> {
    compile_sass_internal(config, source.as_ref(), dest.as_ref(), file_path.as_ref())
}

#[cfg(feature = "sass")]
fn compile_sass_internal(config: &Config,
                         source: &Path,
                         dest: &Path,
                         file_path: &Path)
                         -> Result<()> {
    let mut sass_opts = sass_rs::Options::default();
    sass_opts.include_paths = vec![source
                                       .join(&config.sass.import_dir)
                                       .into_os_string()
                                       .into_string()
                                       .unwrap()];
    sass_opts.output_style = match config.sass.style {
        SassOutputStyle::Nested => sass_rs::OutputStyle::Nested,
        SassOutputStyle::Expanded => sass_rs::OutputStyle::Expanded,
        SassOutputStyle::Compact => sass_rs::OutputStyle::Compact,
        SassOutputStyle::Compressed => sass_rs::OutputStyle::Compressed,
    };

    let src_file = source.join(file_path);
    let content = sass_rs::compile_file(src_file.as_path(), sass_opts.clone())?;
    let mut dest_file = dest.join(file_path);
    dest_file.set_extension("css");

    create_document_file(content, dest_file)
}

#[cfg(not(feature = "sass"))]
fn compile_sass_internal(_config: &Config,
                         source: &Path,
                         dest: &Path,
                         file_path: &Path)
                         -> Result<()> {
    let src_file = source.join(file_path);
    copy_file(src_file.as_path(), dest.join(file_path).as_path())
}

fn copy_file(src_file: &Path, dest_file: &Path) -> Result<()> {
    // create target directories if any exist
    if let Some(parent) = dest_file.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("Could not create {:?}: {}", parent, e))?;
    }

    debug!("Copying {:?} to {:?}", src_file, dest_file);
    fs::copy(src_file, dest_file)
        .map_err(|e| format!("Could not copy {:?} into {:?}: {}", src_file, dest_file, e))?;
    Ok(())
}

fn create_document_file<S: AsRef<str>, P: AsRef<Path>>(content: S, dest_file: P) -> Result<()> {
    create_document_file_internal(content.as_ref(), dest_file.as_ref())
}

fn create_document_file_internal(content: &str, dest_file: &Path) -> Result<()> {
    // create target directories if any exist
    if let Some(parent) = dest_file.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("Could not create {:?}: {}", parent, e))?;
    }

    let mut file = File::create(dest_file)
        .map_err(|e| format!("Could not create {:?}: {}", dest_file, e))?;

    file.write_all(content.as_bytes())?;
    info!("Created {}", dest_file.display());
    Ok(())
}
