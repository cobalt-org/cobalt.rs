use std::fs;
use std::collections::HashMap;
use std::io::Write;
use std::path::Path;
use std::ffi::OsStr;
use liquid;
use rss;
use jsonfeed::Feed;
use jsonfeed;

use cobalt_model::{Config, SortOrder};
use cobalt_model::files;
use cobalt_model;
use document::Document;
use error::*;
use template;

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

    let parser = template::LiquidParser::with_config(config)?;

    let mut documents = vec![];

    let mut page_files = files::FilesBuilder::new(source)?;
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
        let rel_src = file_path
            .strip_prefix(source)
            .expect("file was found under the root");

        // if the document is in the posts folder it's considered a post
        let is_post = file_path.starts_with(posts_path.as_path());
        let default_front = if is_post {
            config.posts.default.clone()
        } else {
            config.pages.default.clone()
        };

        let doc = Document::parse(&file_path, rel_src, default_front)
            .chain_err(|| format!("Failed to parse {:?}", rel_src))?;
        if !doc.front.is_draft || config.include_drafts {
            documents.push(doc);
        }
    }

    if config.include_drafts {
        if let Some(ref drafts_dir) = config.posts.drafts_dir {
            debug!("Draft directory: {:?}", drafts_dir);
            let drafts_root = source.join(&drafts_dir);
            let mut draft_files = files::FilesBuilder::new(drafts_root.as_path())?;
            for line in &config.ignore {
                draft_files.add_ignore(line.as_str())?;
            }
            let draft_files = draft_files.build()?;
            for file_path in draft_files.files().filter(|p| {
                template_extensions.contains(&p.extension().unwrap_or_else(|| OsStr::new("")))
            }) {
                // Provide a fake path as if it was not a draft
                let rel_src = file_path
                    .strip_prefix(&drafts_root)
                    .expect("file was found under the root");
                let new_path = Path::new(&config.posts.dir).join(rel_src);

                let default_front = config.posts.default.clone().set_draft(true);

                let doc = Document::parse(&file_path, &new_path, default_front)
                    .chain_err(|| format!("Failed to parse {:?}", rel_src))?;
                documents.push(doc);
            }
        }
    }

    // January 1, 1970 0:00:00 UTC, the beginning of time
    let default_date = cobalt_model::DateTime::default();

    let (mut posts, documents): (Vec<Document>, Vec<Document>) =
        documents
            .into_iter()
            .partition(|x| x.front.collection == "posts");

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
        let previous = simple_posts_data
            .get(i + 1)
            .cloned()
            .unwrap_or(liquid::Value::Nil);
        post.attributes.insert("previous".to_owned(), previous);
        let next = if i >= 1 {
            simple_posts_data.get(i - 1)
        } else {
            None
        }.cloned()
            .unwrap_or(liquid::Value::Nil);
        post.attributes.insert("next".to_owned(), next);

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
            files::write_document_file(content, dest.join(file_path))?;
        }

        // Everything done with `globals` is terrible for performance.  liquid#95 allows us to
        // improve this.
        let mut posts_variable = config.posts.attributes.clone();
        posts_variable.insert("pages".to_owned(),
                              liquid::Value::Array(simple_posts_data.clone()));
        let global_collection: liquid::Object = vec![(config.posts.slug.clone(),
                                                      liquid::Value::Object(posts_variable))]
            .into_iter()
            .collect();
        let mut globals: liquid::Object =
            vec![("site".to_owned(), liquid::Value::Object(config.site.attributes.clone())),
                 ("collections".to_owned(), liquid::Value::Object(global_collection))]
                .into_iter()
                .collect();
        globals.insert("page".to_owned(),
                       liquid::Value::Object(post.attributes.clone()));
        post.render_excerpt(&globals, &parser, &config.syntax_highlight.theme)
            .chain_err(|| format!("Failed to render excerpt for {:?}", post.file_path))?;
        post.render_content(&globals, &parser, &config.syntax_highlight.theme)
            .chain_err(|| format!("Failed to render content for {:?}", post.file_path))?;

        // Refresh `page` with the `excerpt` / `content` attribute
        globals.insert("page".to_owned(),
                       liquid::Value::Object(post.attributes.clone()));
        let post_html = post.render(&globals, &parser, &layouts, &mut layouts_cache)
            .chain_err(|| format!("Failed to render for {:?}", post.file_path))?;
        files::write_document_file(post_html, dest.join(&post.file_path))?;
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
            files::write_document_file(content, dest.join(file_path))?;
        }

        let mut posts_variable = config.posts.attributes.clone();
        posts_variable.insert("pages".to_owned(), liquid::Value::Array(posts_data.clone()));
        let global_collection: liquid::Object = vec![(config.posts.slug.clone(),
                                                      liquid::Value::Object(posts_variable))]
            .into_iter()
            .collect();
        let mut globals: liquid::Object =
            vec![("site".to_owned(), liquid::Value::Object(config.site.attributes.clone())),
                 ("collections".to_owned(), liquid::Value::Object(global_collection))]
                .into_iter()
                .collect();
        globals.insert("page".to_owned(),
                       liquid::Value::Object(doc.attributes.clone()));
        doc.render_excerpt(&globals, &parser, &config.syntax_highlight.theme)
            .chain_err(|| format!("Failed to render excerpt for {:?}", doc.file_path))?;
        doc.render_content(&globals, &parser, &config.syntax_highlight.theme)
            .chain_err(|| format!("Failed to render content for {:?}", doc.file_path))?;

        // Refresh `page` with the `excerpt` / `content` attribute
        globals.insert("page".to_owned(),
                       liquid::Value::Object(doc.attributes.clone()));
        let doc_html = doc.render(&globals, &parser, &layouts, &mut layouts_cache)
            .chain_err(|| format!("Failed to render for {:?}", doc.file_path))?;
        files::write_document_file(doc_html, dest.join(doc.file_path))?;
    }

    // copy all remaining files in the source to the destination
    // compile SASS along the way
    {
        debug!("Copying remaining assets");

        config.assets.populate(dest)?;
    }

    Ok(())
}

// creates a new RSS file with the contents of the site blog
fn create_rss(path: &str, dest: &Path, config: &Config, posts: &[Document]) -> Result<()> {
    let rss_path = dest.join(path);
    debug!("Creating RSS file at {}", rss_path.display());

    let title = &config.posts.title;
    let description = config
        .posts
        .description
        .as_ref()
        .map(|s| s.as_str())
        .unwrap_or("");
    let link = config
        .site
        .base_url
        .as_ref()
        .ok_or(ErrorKind::ConfigFileMissingFields)?;

    let items: Result<Vec<rss::Item>> = posts.iter().map(|doc| doc.to_rss(link)).collect();
    let items = items?;

    let channel = rss::ChannelBuilder::default()
        .title(title.to_owned())
        .link(link.to_owned())
        .description(description.to_owned())
        .items(items)
        .build()?;

    let rss_string = channel.to_string();
    trace!("RSS data: {}", rss_string);

    // create target directories if any exist
    if let Some(parent) = rss_path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("Could not create {:?}: {}", parent, e))?;
    }

    let mut rss_file = fs::File::create(&rss_path)?;
    rss_file
        .write_all(br#"<?xml version="1.0" encoding="UTF-8"?>"#)?;
    rss_file.write_all(&rss_string.into_bytes())?;
    rss_file.write_all(b"\n")?;

    Ok(())
}

// creates a new jsonfeed file with the contents of the site blog
fn create_jsonfeed(path: &str, dest: &Path, config: &Config, posts: &[Document]) -> Result<()> {
    let jsonfeed_path = dest.join(path);
    debug!("Creating jsonfeed file at {}", jsonfeed_path.display());

    let title = &config.posts.title;
    let description = config
        .posts
        .description
        .as_ref()
        .map(|s| s.as_str())
        .unwrap_or("");
    let link = config
        .site
        .base_url
        .as_ref()
        .ok_or(ErrorKind::ConfigFileMissingFields)?;

    let jsonitems = posts.iter().map(|doc| doc.to_jsonfeed(link)).collect();

    let feed = Feed {
        title: title.to_string(),
        items: jsonitems,
        home_page_url: Some(link.to_string()),
        description: Some(description.to_string()),
        ..Default::default()
    };

    let jsonfeed_string = jsonfeed::to_string(&feed).unwrap();
    files::write_document_file(jsonfeed_string, jsonfeed_path)?;

    Ok(())
}
