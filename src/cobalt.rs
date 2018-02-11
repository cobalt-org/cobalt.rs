use std::fs;
use std::collections::HashMap;
use std::io::Write;
use std::path::{PathBuf, Path};
use liquid;
use rss;
use jsonfeed::Feed;
use jsonfeed;

use cobalt_model::{Config, SortOrder};
use cobalt_model::files;
use cobalt_model::Collection;
use cobalt_model;
use document::Document;
use error::*;
use template;

/// The primary build function that transforms a directory into a site
pub fn build(config: &Config) -> Result<()> {
    trace!("Build configuration: {:?}", config);

    let source = config.source.as_path();
    let dest = config.destination.as_path();

    let parser = template::LiquidParser::with_config(config)?;
    let layouts = source.join(&config.layouts_dir);
    let mut layouts_cache = HashMap::new();
    let posts_path = source.join(&config.posts.dir);

    debug!("Layouts directory: {:?}", layouts);
    debug!("Posts directory: {:?}", posts_path);
    debug!("Draft mode enabled: {}", config.include_drafts);

    let post_files = find_post_files(source, config)?;
    let mut posts = parse_pages(&post_files, &config.posts, source)?;
    process_included_drafts(config, source, &mut posts)?;

    let page_files = find_page_files(source, config)?;
    let documents = parse_pages(&page_files, &config.pages, source)?;

    sort_pages(&mut posts, config)?;
    generate_posts(&mut posts,
                   config,
                   &parser,
                   dest,
                   &layouts,
                   &mut layouts_cache)?;

    // check if we should create an RSS file and create it!
    if let Some(ref path) = config.posts.rss {
        create_rss(path, dest, config, &posts)?;
    }
    // check if we should create an jsonfeed file and create it!
    if let Some(ref path) = config.posts.jsonfeed {
        create_jsonfeed(path, dest, config, &posts)?;
    }

    generate_pages(posts,
                   documents,
                   config,
                   &parser,
                   dest,
                   &layouts,
                   &mut layouts_cache)?;

    // copy all remaining files in the source to the destination
    // compile SASS along the way
    {
        debug!("Copying remaining assets");
        config.assets.populate(dest)?;
    }

    Ok(())
}

fn generate_doc(dest: &Path,
                layouts: &Path,
                mut layouts_cache: &mut HashMap<String, String>,
                parser: &template::LiquidParser,
                posts_data: &[liquid::Value],
                doc: &mut Document,
                config: &Config)
                -> Result<()> {
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

    // Everything done with `globals` is terrible for performance.  liquid#95 allows us to
    // improve this.
    let mut posts_variable = config.posts.attributes.clone();
    posts_variable.insert("pages".to_owned(),
                          liquid::Value::Array(posts_data.to_vec()));
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

    doc.render_excerpt(&globals, parser, &config.syntax_highlight.theme)
        .chain_err(|| format!("Failed to render excerpt for {:?}", doc.file_path))?;
    doc.render_content(&globals, parser, &config.syntax_highlight.theme)
        .chain_err(|| format!("Failed to render content for {:?}", doc.file_path))?;

    // Refresh `page` with the `excerpt` / `content` attribute
    globals.insert("page".to_owned(),
                   liquid::Value::Object(doc.attributes.clone()));
    let doc_html = doc.render(&globals, parser, layouts, &mut layouts_cache)
        .chain_err(|| format!("Failed to render for {:?}", doc.file_path))?;
    files::write_document_file(doc_html, dest.join(&doc.file_path))?;
    Ok(())
}

fn generate_pages(posts: Vec<Document>,
                  documents: Vec<Document>,
                  config: &Config,
                  parser: &template::LiquidParser,
                  dest: &Path,
                  layouts: &Path,
                  mut layouts_cache: &mut HashMap<String, String>)
                  -> Result<()> {
    // during post rendering additional attributes such as content were
    // added to posts. collect them so that non-post documents can access them
    let posts_data: Vec<liquid::Value> = posts
        .into_iter()
        .map(|x| liquid::Value::Object(x.attributes))
        .collect();

    trace!("Generating other documents");
    for mut doc in documents {
        trace!("Generating {}", doc.url_path);
        generate_doc(dest,
                     layouts,
                     &mut layouts_cache,
                     parser,
                     &posts_data,
                     &mut doc,
                     config)?;
    }

    Ok(())
}

fn generate_posts(posts: &mut Vec<Document>,
                  config: &Config,
                  parser: &template::LiquidParser,
                  dest: &Path,
                  layouts: &Path,
                  mut layouts_cache: &mut HashMap<String, String>)
                  -> Result<()> {
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

        generate_doc(dest,
                     layouts,
                     &mut layouts_cache,
                     parser,
                     &simple_posts_data,
                     post,
                     config)?;
    }

    Ok(())
}

fn sort_pages(posts: &mut Vec<Document>, config: &Config) -> Result<()> {
    // January 1, 1970 0:00:00 UTC, the beginning of time
    let default_date = cobalt_model::DateTime::default();

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
        SortOrder::None => (),
    }

    Ok(())
}

fn parse_drafts(drafts_root: &PathBuf,
                draft_files: &files::Files,
                documents: &mut Vec<Document>,
                config: &Config)
                -> Result<()> {
    for file_path in draft_files.files() {
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
    Ok(())
}

fn process_included_drafts(config: &Config,
                           source: &Path,
                           documents: &mut Vec<Document>)
                           -> Result<()> {
    if config.include_drafts {
        if let Some(ref drafts_dir) = config.posts.drafts_dir {
            debug!("Draft directory: {:?}", drafts_dir);
            let drafts_root = source.join(&drafts_dir);
            let draft_files = find_post_draft_files(&drafts_root, config)?;

            parse_drafts(&drafts_root, &draft_files, documents, config)?;
        }
    }
    Ok(())
}

fn find_post_files(source: &Path, config: &Config) -> Result<files::Files> {
    let mut page_files = files::FilesBuilder::new(source)?;
    page_files
        .add_ignore(&format!("!/{}", config.posts.dir))?
        .add_ignore(&format!("!/{}/**", config.posts.dir))?
        .add_ignore(&format!("/{}/**/_*", config.posts.dir))?
        .add_ignore(&format!("/{}/**/_*/**", config.posts.dir))?;
    for line in &config.posts.ignore {
        page_files.add_ignore(line.as_str())?;
    }
    for ext in config.posts.template_extensions.iter() {
        page_files.add_extension(ext)?;
    }
    page_files.limit(PathBuf::from(&config.posts.dir))?;
    page_files.build()
}

fn find_post_draft_files(drafts_root: &Path, config: &Config) -> Result<files::Files> {
    let mut page_files = files::FilesBuilder::new(drafts_root)?;
    for line in &config.posts.ignore {
        page_files.add_ignore(line.as_str())?;
    }
    for ext in config.posts.template_extensions.iter() {
        page_files.add_extension(ext)?;
    }
    page_files.build()
}

fn find_page_files(source: &Path, config: &Config) -> Result<files::Files> {
    let mut page_files = files::FilesBuilder::new(source)?;
    for line in &config.pages.ignore {
        page_files.add_ignore(line.as_str())?;
    }
    for ext in config.pages.template_extensions.iter() {
        page_files.add_extension(ext)?;
    }
    page_files.add_ignore(&format!("/{}", config.posts.dir))?;
    if let Some(ref drafts_dir) = config.posts.drafts_dir {
        page_files.add_ignore(&format!("/{}", drafts_dir))?;
    }
    page_files.build()
}

fn parse_pages(page_files: &files::Files,
               collection: &Collection,
               source: &Path)
               -> Result<Vec<Document>> {
    let mut documents = vec![];
    for file_path in page_files.files() {
        let rel_src = file_path
            .strip_prefix(source)
            .expect("file was found under the root");

        let default_front = collection.default.clone();

        let doc = Document::parse(&file_path, rel_src, default_front)
            .chain_err(|| format!("Failed to parse {:?}", rel_src))?;
        if !doc.front.is_draft || collection.include_drafts {
            documents.push(doc);
        }
    }
    Ok(documents)
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
