use std::fs::{self, File};
use std::collections::HashMap;
use std::io::Write;
use std::path::Path;
use std::ffi::OsStr;
use liquid;
use rss;
use jsonfeed::Feed;
use jsonfeed;

#[cfg(feature = "sass")]
use sass_rs;

use datetime;
use document::Document;
use error::*;
use config::{Config, SortOrder};
#[cfg(feature = "sass")]
use config::SassOutputStyle;
use files::FilesBuilder;

/// The primary build function that transforms a directory into a site
pub fn build(config: &Config) -> Result<()> {
    trace!("Build configuration: {:?}", config);

    let source = config.source.as_path();
    let dest = config.destination.as_path();

    let template_extensions: Vec<&OsStr> =
        config.template_extensions.iter().map(OsStr::new).collect();

    let layouts = source.join(&config.layouts_dir);
    let mut layouts_cache = HashMap::new();
    let posts_path = source.join(&config.posts.dir);

    debug!("Layouts directory: {:?}", layouts);
    debug!("Posts directory: {:?}", posts_path);
    debug!("Draft mode enabled: {}", config.include_drafts);

    let mut documents = vec![];

    let mut page_files = FilesBuilder::new(source)?;
    page_files
        .add_ignore(&format!("!{}", config.posts.dir))?
        .add_ignore(&format!("!{}/**", config.posts.dir))?
        .add_ignore(&format!("{}/**/_*", config.posts.dir))?
        .add_ignore(&format!("{}/**/_*/**", config.posts.dir))?;
    for line in &config.ignore {
        page_files.add_ignore(line.as_str())?;
    }
    let page_files = page_files.build()?;
    for file_path in page_files.files().filter(|p| {
        template_extensions.contains(&p.extension().unwrap_or_else(|| OsStr::new("")))
    }) {
        // if the document is in the posts folder it's considered a post
        let src_path = source.join(file_path.as_path());
        let is_post = src_path.starts_with(posts_path.as_path());

        let default_front = if is_post {
            config.posts.default.clone()
        } else {
            config.pages.default.clone()
        };

        let doc = Document::parse(source, &file_path, &file_path, default_front)
            .chain_err(|| format!("Failed to parse {:?}", src_path))?;
        if !doc.front.is_draft || config.include_drafts {
            documents.push(doc);
        }
    }

    if config.include_drafts {
        if let Some(ref drafts_dir) = config.posts.drafts_dir {
            debug!("Draft directory: {:?}", drafts_dir);
            let drafts_root = source.join(&drafts_dir);
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

                let default_front = config.posts.default.clone().set_draft(true);

                let doc = Document::parse(&drafts_root, &file_path, new_path, default_front)
                    .chain_err(|| {
                                   let src_path = drafts_root.join(file_path);
                                   format!("Failed to parse {:?}", src_path)
                               })?;
                documents.push(doc);
            }
        }
    }

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

    match config.posts.order {
        SortOrder::Asc => posts.reverse(),
        SortOrder::Desc => (),
    }

    // collect all posts attributes to pass them to other posts for rendering
    let simple_posts_data: Vec<liquid::Value> = posts
        .iter()
        .map(|x| liquid::Value::Object(x.attributes.clone()))
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
        context.set_val("site",
                        liquid::Value::Object(config.site.attributes.clone()));

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
    if let Some(ref path) = config.posts.rss {
        create_rss(path, dest, config, &posts)?;
    }
    // check if we should create an jsonfeed file and create it!
    if let Some(ref path) = config.posts.jsonfeed {
        create_jsonfeed(path, dest, config, &posts)?;
    }

    // during post rendering additional attributes such as content were
    // added to posts. collect them so that non-post documents can access them
    let posts_data: Vec<liquid::Value> = posts
        .into_iter()
        .map(|x| liquid::Value::Object(x.attributes))
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
        context.set_val("site",
                        liquid::Value::Object(config.site.attributes.clone()));

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
        .posts
        .name
        .as_ref()
        .or_else(|| config.site.name.as_ref())
        .ok_or(ErrorKind::ConfigFileMissingFields)?;
    let description = config
        .posts
        .description
        .as_ref()
        .or_else(|| config.site.description.as_ref())
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
