use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::path;

use failure::ResultExt;
use jsonfeed;
use jsonfeed::Feed;
use liquid;
use rss;
use sitemap::writer::SiteMapWriter;

use crate::cobalt_model;
use crate::cobalt_model::files;
use crate::cobalt_model::permalink;
use crate::cobalt_model::Collection;
use crate::cobalt_model::{Config, Minify, SortOrder};
use crate::document::{Document, RenderContex};
use crate::error::*;
use crate::pagination;

struct Context {
    pub source: path::PathBuf,
    pub destination: path::PathBuf,
    pub pages: cobalt_model::Collection,
    pub posts: cobalt_model::Collection,
    pub site: liquid::Object,
    pub layouts: HashMap<String, String>,
    pub liquid: cobalt_model::Liquid,
    pub markdown: cobalt_model::Markdown,
    pub assets: cobalt_model::Assets,
    pub sitemap: Option<String>,
    pub minify: Minify,
}

impl Context {
    fn with_config(config: Config) -> Result<Self> {
        let Config {
            source,
            destination,
            pages,
            posts,
            site,
            layouts_dir,
            liquid,
            markdown,
            assets,
            sitemap,
            minify,
        } = config;

        let pages = pages.build()?;
        let posts = posts.build()?;
        let site = site.build()?;
        let liquid = liquid.build()?;
        let markdown = markdown.build();
        let assets = assets.build()?;

        let layouts = find_layouts(&layouts_dir)?;
        let layouts = parse_layouts(&layouts);

        let context = Context {
            source,
            destination,
            pages,
            posts,
            site,
            layouts,
            liquid,
            markdown,
            assets,
            sitemap,
            minify,
        };
        Ok(context)
    }
}

/// The primary build function that transforms a directory into a site
pub fn build(config: Config) -> Result<()> {
    let context = Context::with_config(config)?;

    let post_files = &context.posts.pages;
    let mut posts = parse_pages(post_files, &context.posts, &context.source)?;
    if let Some(ref drafts) = context.posts.drafts {
        let drafts_root = drafts.subtree();
        parse_drafts(drafts_root, drafts, &mut posts, &context.posts)?;
    }

    let page_files = &context.pages.pages;
    let documents = parse_pages(page_files, &context.pages, &context.source)?;

    sort_pages(&mut posts, &context.posts)?;
    generate_posts(&mut posts, &context)?;

    // check if we should create an RSS file and create it!
    if let Some(ref path) = context.posts.rss {
        create_rss(path, &context.destination, &context.posts, &posts)?;
    }
    // check if we should create an jsonfeed file and create it!
    if let Some(ref path) = context.posts.jsonfeed {
        create_jsonfeed(path, &context.destination, &context.posts, &posts)?;
    }
    if let Some(ref path) = context.sitemap {
        let sitemap_path = &context.destination.join(path);
        create_sitemap(
            &sitemap_path,
            &context.posts,
            &posts,
            &context.pages,
            &documents,
        )?;
    }

    generate_pages(posts, documents, &context)?;

    // copy all remaining files in the source to the destination
    // compile SASS along the way
    context
        .assets
        .populate(&context.destination, &context.minify)?;

    Ok(())
}

fn generate_collections_var(
    posts_data: &[liquid::model::Value],
    context: &Context,
) -> (kstring::KString, liquid::model::Value) {
    let mut posts_variable = context.posts.attributes.clone();
    posts_variable.insert(
        "pages".into(),
        liquid::model::Value::Array(posts_data.to_vec()),
    );
    let global_collection: liquid::Object = vec![(
        context.posts.slug.clone().into(),
        liquid::model::Value::Object(posts_variable),
    )]
    .into_iter()
    .collect();
    (
        "collections".into(),
        liquid::model::Value::Object(global_collection),
    )
}

fn generate_doc(
    doc: &mut Document,
    context: &Context,
    global_collection: (kstring::KString, liquid::model::Value),
) -> Result<()> {
    // Everything done with `globals` is terrible for performance.  liquid#95 allows us to
    // improve this.
    let mut globals: liquid::Object = vec![
        (
            "site".into(),
            liquid::model::Value::Object(context.site.clone()),
        ),
        global_collection,
    ]
    .into_iter()
    .collect();
    globals.insert(
        "page".into(),
        liquid::model::Value::Object(doc.attributes.clone()),
    );
    {
        let render_context = RenderContex {
            parser: &context.liquid,
            markdown: &context.markdown,
            globals: &globals,
            minify: context.minify.clone(),
        };

        doc.render_excerpt(&render_context).with_context(|_| {
            failure::format_err!("Failed to render excerpt for {}", doc.file_path.display())
        })?;
        doc.render_content(&render_context).with_context(|_| {
            failure::format_err!("Failed to render content for {}", doc.file_path.display())
        })?;
    }

    // Refresh `page` with the `excerpt` / `content` attribute
    globals.insert(
        "page".into(),
        liquid::model::Value::Object(doc.attributes.clone()),
    );
    let render_context = RenderContex {
        parser: &context.liquid,
        markdown: &context.markdown,
        globals: &globals,
        minify: context.minify.clone(),
    };
    let doc_html = doc
        .render(&render_context, &context.layouts)
        .with_context(|_| {
            failure::format_err!("Failed to render for {}", doc.file_path.display())
        })?;
    files::write_document_file(doc_html, context.destination.join(&doc.file_path))?;
    Ok(())
}

fn generate_pages(posts: Vec<Document>, documents: Vec<Document>, context: &Context) -> Result<()> {
    // during post rendering additional attributes such as content were
    // added to posts. collect them so that non-post documents can access them
    let posts_data: Vec<liquid::model::Value> = posts
        .into_iter()
        .map(|x| liquid::model::Value::Object(x.attributes))
        .collect();

    trace!("Generating other documents");
    for mut doc in documents {
        trace!("Generating {}", doc.url_path);
        if doc.front.pagination.is_some() {
            let paginators = pagination::generate_paginators(&mut doc, &posts_data)?;
            // page 1 uses frontmatter.permalink instead of paginator.permalink
            let mut paginators = paginators.into_iter();
            let paginator = paginators
                .next()
                .expect("We detected pagination enabled but we have no paginator");
            generate_doc(
                &mut doc,
                context,
                (
                    "paginator".into(),
                    liquid::model::Value::Object(paginator.into()),
                ),
            )?;
            for paginator in paginators {
                let mut doc_page = doc.clone();
                doc_page.file_path = permalink::format_url_as_file(&paginator.index_permalink);
                generate_doc(
                    &mut doc_page,
                    context,
                    (
                        "paginator".into(),
                        liquid::model::Value::Object(paginator.into()),
                    ),
                )?;
            }
        } else {
            generate_doc(
                &mut doc,
                context,
                generate_collections_var(&posts_data, &context),
            )?;
        };
    }
    Ok(())
}

fn generate_posts(posts: &mut Vec<Document>, context: &Context) -> Result<()> {
    // collect all posts attributes to pass them to other posts for rendering
    let simple_posts_data: Vec<liquid::model::Value> = posts
        .iter()
        .map(|x| liquid::model::Value::Object(x.attributes.clone()))
        .collect();

    trace!("Generating posts");
    for (i, post) in &mut posts.iter_mut().enumerate() {
        trace!("Generating {}", post.url_path);

        // posts are in reverse date order, so previous post is the next in the list (+1)
        let previous = simple_posts_data
            .get(i + 1)
            .cloned()
            .unwrap_or(liquid::model::Value::Nil);
        post.attributes.insert("previous".into(), previous);

        let next = if i >= 1 {
            simple_posts_data.get(i - 1)
        } else {
            None
        }
        .cloned()
        .unwrap_or(liquid::model::Value::Nil);
        post.attributes.insert("next".into(), next);

        generate_doc(
            post,
            context,
            generate_collections_var(&simple_posts_data, &context),
        )?;
    }

    Ok(())
}

fn sort_pages(posts: &mut Vec<Document>, collection: &Collection) -> Result<()> {
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

    match collection.order {
        SortOrder::Asc => posts.reverse(),
        SortOrder::Desc | SortOrder::None => (),
    }

    Ok(())
}

fn parse_drafts(
    drafts_root: &path::Path,
    draft_files: &files::Files,
    documents: &mut Vec<Document>,
    collection: &Collection,
) -> Result<()> {
    let rel_real = collection
        .pages
        .subtree()
        .strip_prefix(collection.pages.root())
        .expect("subtree is under root");
    for file_path in draft_files.files() {
        // Provide a fake path as if it was not a draft
        let rel_src = file_path
            .strip_prefix(&drafts_root)
            .expect("file was found under the root");
        let new_path = rel_real.join(rel_src);

        let default_front = collection.default.clone().set_draft(true);

        let doc = Document::parse(&file_path, &new_path, default_front)
            .with_context(|_| failure::format_err!("Failed to parse {}", rel_src.display()))?;
        documents.push(doc);
    }
    Ok(())
}

fn find_layouts(layouts: &path::Path) -> Result<files::Files> {
    let mut files = files::FilesBuilder::new(layouts)?;
    files.ignore_hidden(false)?;
    files.build()
}

fn parse_layouts(files: &files::Files) -> HashMap<String, String> {
    let (entries, errors): (Vec<_>, Vec<_>) = files
        .files()
        .map(|file_path| {
            let rel_src = file_path
                .strip_prefix(files.root())
                .expect("file was found under the root");

            let layout_data = files::read_file(&file_path).with_context(|_| {
                failure::format_err!("Failed to load layout {}", rel_src.display())
            })?;

            let path = rel_src
                .to_str()
                .ok_or_else(|| {
                    failure::format_err!("File name not valid liquid path: {}", rel_src.display())
                })?
                .to_owned();

            Ok((path, layout_data))
        })
        .partition(Result::is_ok);

    for error in errors {
        warn!("{}", error.expect_err("partition to filter out oks"));
    }

    entries
        .into_iter()
        .map(|entry| entry.expect("partition to filter out errors"))
        .collect()
}

fn parse_pages(
    page_files: &files::Files,
    collection: &Collection,
    source: &path::Path,
) -> Result<Vec<Document>> {
    let mut documents = vec![];
    for file_path in page_files.files() {
        let rel_src = file_path
            .strip_prefix(source)
            .expect("file was found under the root");

        let default_front = collection.default.clone();

        let doc = Document::parse(&file_path, rel_src, default_front)
            .with_context(|_| failure::format_err!("Failed to parse {}", rel_src.display()))?;
        if !doc.front.is_draft || collection.include_drafts {
            documents.push(doc);
        }
    }
    Ok(documents)
}

// creates a new RSS file with the contents of the site blog
fn create_rss(
    path: &str,
    dest: &path::Path,
    collection: &Collection,
    documents: &[Document],
) -> Result<()> {
    let rss_path = dest.join(path);
    debug!("Creating RSS file at {}", rss_path.display());

    let title = &collection.title;
    let description = collection.description.as_deref().unwrap_or("");
    let link = collection
        .base_url
        .as_ref()
        .ok_or_else(|| failure::err_msg("`base_url` is required for RSS support"))?;

    let items: Result<Vec<rss::Item>> = documents.iter().map(|doc| doc.to_rss(link)).collect();
    let items = items?;

    let channel = rss::ChannelBuilder::default()
        .title(title.to_owned())
        .link(link.to_owned())
        .description(description.to_owned())
        .items(items)
        .build()
        .map_err(failure::err_msg)?;

    let rss_string = channel.to_string();
    trace!("RSS data: {}", rss_string);

    // create target directories if any exist
    if let Some(parent) = rss_path.parent() {
        fs::create_dir_all(parent)
            .with_context(|_| failure::format_err!("Could not create {}", parent.display()))?;
    }

    let mut rss_file = fs::File::create(&rss_path)?;
    rss_file.write_all(&rss_string.into_bytes())?;
    rss_file.write_all(b"\n")?;

    Ok(())
}

// creates a new jsonfeed file with the contents of the site blog
fn create_jsonfeed(
    path: &str,
    dest: &path::Path,
    collection: &Collection,
    documents: &[Document],
) -> Result<()> {
    let jsonfeed_path = dest.join(path);
    debug!("Creating jsonfeed file at {}", jsonfeed_path.display());

    let title = &collection.title;
    let description = collection.description.as_deref().unwrap_or("");
    let link = collection
        .base_url
        .as_ref()
        .ok_or_else(|| failure::err_msg("`base_url` is required for jsonfeed support"))?;

    let jsonitems = documents.iter().map(|doc| doc.to_jsonfeed(link)).collect();

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
fn create_sitemap(
    sitemap_path: &path::Path,
    collection: &Collection,
    documents: &[Document],
    collection_pages: &Collection,
    documents_pages: &[Document],
) -> Result<()> {
    debug!("Creating sitemap file at {}", sitemap_path.display());
    let mut buff = Vec::new();
    let writer = SiteMapWriter::new(&mut buff);
    let link = collection
        .base_url
        .as_ref()
        .ok_or_else(|| failure::err_msg("`base_url` is required for sitemap support"))?;
    let mut urls = writer.start_urlset()?;
    for doc in documents {
        doc.to_sitemap(link, &mut urls)?;
    }

    let link = collection_pages
        .base_url
        .as_ref()
        .ok_or_else(|| failure::err_msg("`base_url` is required for sitemap support"))?;
    for doc in documents_pages {
        doc.to_sitemap(link, &mut urls)?;
    }
    urls.end()?;

    files::write_document_file(String::from_utf8(buff)?, sitemap_path)?;

    Ok(())
}
