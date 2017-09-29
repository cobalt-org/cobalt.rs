use std::fs::{self, File};
use std::collections::HashMap;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::ffi::OsStr;
use liquid::Value;
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
use frontmatter;

/// The primary build function that transforms a directory into a site
pub fn build(config: &Config) -> Result<()> {
    trace!("Build configuration: {:?}", config);

    let source = Path::new(&config.source);
    let dest = Path::new(&config.dest);

    let template_extensions: Vec<&OsStr> = config
        .template_extensions
        .iter()
        .map(OsStr::new)
        .collect();

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
    for file_path in page_files
            .files()
            .filter(|p| {
                        template_extensions
                            .contains(&p.extension().unwrap_or_else(|| OsStr::new("")))
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

        let doc = Document::parse(source, &file_path, &file_path, default_front)?;
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
        for file_path in draft_files
                .files()
                .filter(|p| {
                            template_extensions
                                .contains(&p.extension().unwrap_or_else(|| OsStr::new("")))
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

            let doc = Document::parse(&drafts_root, &file_path, new_path, default_front)?;
            documents.push(doc);
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
            create_document_file(content, &file_path, dest)?;
        }

        let mut context = post.get_render_context(&simple_posts_data);

        post.render_excerpt(&mut context, source, &config.syntax_highlight.theme)
            .chain_err(|| format!("Failed to render excerpt for {:?}", post.file_path))?;
        let post_html = post.render(&mut context,
                                    source,
                                    &layouts,
                                    &mut layouts_cache,
                                    &config.syntax_highlight.theme)
            .chain_err(|| format!("Failed to render for {:?}", post.file_path))?;
        create_document_file(post_html, &post.file_path, dest)?;
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
            create_document_file(content, &file_path, dest)?;
        }

        let mut context = doc.get_render_context(&posts_data);
        let doc_html = doc.render(&mut context,
                                  source,
                                  &layouts,
                                  &mut layouts_cache,
                                  &config.syntax_highlight.theme)
            .chain_err(|| format!("Failed to render for {:?}", doc.file_path))?;
        create_document_file(doc_html, doc.file_path, dest)?;
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
        for file_path in asset_files
                .files()
                .filter(|p| {
                            !template_extensions
                                 .contains(&p.extension().unwrap_or_else(|| OsStr::new("")))
                        }) {
            {
                let parent_dir = file_path.parent();
                if let Some(parent_dir) = parent_dir {
                    let parent_dir = dest.join(parent_dir);
                    debug!("Creating new directory {:?}", parent_dir);
                    fs::create_dir_all(parent_dir)?;
                }
            }

            #[cfg(feature = "sass")]
            {
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

                let src_file = source.join(file_path.as_path());
                if file_path.extension() == Some(OsStr::new("scss")) {
                    let content = sass_rs::compile_file(src_file.as_path(), sass_opts.clone())?;
                    let mut dest_file = dest.join(file_path.clone());
                    dest_file.set_extension("css");

                    let mut file =
                        File::create(&dest_file)
                            .map_err(|e| format!("Could not create {:?}: {}", file_path, e))?;

                    file.write_all(content.as_bytes())?;

                } else {
                    copy_file(src_file.as_path(), dest.join(file_path).as_path())?;
                }
            }

            #[cfg(not(feature = "sass"))]
            {

                let src_file = source.join(file_path.as_path());
                copy_file(src_file.as_path(), dest.join(file_path).as_path())?;
            }
        }
    }

    Ok(())
}

fn copy_file(src_file: &Path, dest_file: &Path) -> Result<()> {
    debug!("Copying {:?} to {:?}", src_file, dest_file);
    fs::copy(src_file, dest_file)
        .map_err(|e| format!("Could not copy {:?} into {:?}: {}", src_file, dest_file, e))?;
    Ok(())
}

// creates a new RSS file with the contents of the site blog
fn create_rss(path: &str, dest: &Path, config: &Config, posts: &[Document]) -> Result<()> {
    match (&config.name, &config.description, &config.link) {
        // these three fields are mandatory in the RSS standard
        (&Some(ref name), &Some(ref description), &Some(ref link)) => {
            trace!("Generating RSS data");

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
        _ => Err(ErrorKind::ConfigFileMissingFields.into()),
    }
}
// creates a new jsonfeed file with the contents of the site blog
fn create_jsonfeed(path: &str, dest: &Path, config: &Config, posts: &[Document]) -> Result<()> {
    match (&config.name, &config.description, &config.link) {
        (&Some(ref name), &Some(ref desc), &Some(ref link)) => {
            trace!("Generating jsonfeed data");

            let jsonitems = posts.iter().map(|doc| doc.to_jsonfeed(link)).collect();

            let feed = Feed {
                title: name.to_string(),
                items: jsonitems,
                home_page_url: Some(link.to_string()),
                description: Some(desc.to_string()),
                ..Default::default()
            };

            let jsonfeed_string = jsonfeed::to_string(&feed).unwrap();
            let jsonfeed_path = dest.join(path);
            let mut jsonfeed_file = File::create(&jsonfeed_path)?;
            jsonfeed_file.write_all(&jsonfeed_string.into_bytes())?;

            info!("Created jsonfeed file at {}", jsonfeed_path.display());
            Ok(())
        }
        _ => Err(ErrorKind::ConfigFileMissingFields.into()),
    }
}

fn create_document_file<S: AsRef<str>, T: AsRef<Path>, R: Into<PathBuf>>(content: S,
                                                                         relpath: T,
                                                                         dest: R)
                                                                         -> Result<()> {
    create_document_file_internal(content.as_ref(), relpath.as_ref(), dest.into())
}

fn create_document_file_internal(content: &str, relpath: &Path, dest: PathBuf) -> Result<()> {
    // construct target path
    let mut file_path = dest;
    file_path.push(relpath);

    // create target directories if any exist
    if let Some(parent) = file_path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("Could not create {:?}: {}", parent, e))?;
    }

    let mut file = File::create(&file_path)
        .map_err(|e| format!("Could not create {:?}: {}", file_path, e))?;

    file.write_all(content.as_bytes())?;
    info!("Created {}", file_path.display());
    Ok(())
}
