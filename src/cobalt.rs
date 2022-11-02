use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::path;

use failure::ResultExt;
use jsonfeed::Feed;
use log::debug;
use log::trace;
use log::warn;
use sitemap::writer::SiteMapWriter;

use crate::cobalt_model;
use crate::cobalt_model::files;
use crate::cobalt_model::permalink;
use crate::cobalt_model::Collection;
use crate::cobalt_model::{Config, Minify, SortOrder};
use crate::document::{Document, RenderContext};
use crate::error::*;
use crate::pagination;

struct Context {
    pub destination: path::PathBuf,
    pub source_files: cobalt_core::Source,
    pub page_extensions: Vec<liquid::model::KString>,
    pub include_drafts: bool,
    pub pages: cobalt_model::Collection,
    pub posts: cobalt_model::Collection,
    pub site: cobalt_model::Site,
    pub site_attributes: liquid::Object,
    pub layouts: HashMap<String, String>,
    pub liquid: cobalt_model::Liquid,
    pub markdown: cobalt_model::Markdown,
    pub vimwiki: cobalt_model::Vimwiki,
    pub assets: cobalt_model::Assets,
    pub minify: Minify,
}

impl Context {
    fn with_config(config: Config) -> Result<Self> {
        let Config {
            source,
            destination,
            ignore,
            page_extensions,
            include_drafts,
            pages,
            posts,
            site,
            layouts_path,
            liquid,
            markdown,
            vimwiki,
            syntax: _,
            assets,
            minify,
        } = config;

        let source_files = cobalt_core::Source::new(&source, ignore.iter().map(|s| s.as_str()))?;
        let site_attributes = site.load(&source)?;
        let liquid = liquid.build()?;
        let markdown = markdown.build();
        let vimwiki = vimwiki.build();
        let assets = assets.build()?;

        let layouts = find_layouts(&layouts_path)?;
        let layouts = parse_layouts(&layouts);

        let context = Context {
            destination,
            source_files,
            page_extensions,
            include_drafts,
            pages,
            posts,
            site,
            site_attributes,
            layouts,
            liquid,
            markdown,
            vimwiki,
            assets,
            minify,
        };
        Ok(context)
    }
}

/// The primary build function that transforms a directory into a site
pub fn build(config: Config) -> Result<()> {
    let context = Context::with_config(config)?;

    let mut post_paths = Vec::new();
    let mut post_draft_paths = Vec::new();
    let mut page_paths = Vec::new();
    let mut asset_paths = Vec::new();
    for path in context.source_files.iter() {
        match classify_path(
            &path.rel_path,
            &context.pages,
            &context.posts,
            &context.page_extensions,
        ) {
            Some((slug, false)) if context.pages.slug == slug => page_paths.push(path),
            Some((slug, true)) if context.pages.slug == slug => {
                unreachable!("We don't support draft pages")
            }
            Some((slug, false)) if context.posts.slug == slug => post_paths.push(path),
            Some((slug, true)) if context.posts.slug == slug => post_draft_paths.push(path),
            Some((slug, _)) => unreachable!("Unknown collection: {}", slug),
            None => asset_paths.push(path),
        }
    }

    let mut posts = parse_pages(&post_paths, &context.posts, context.include_drafts)?;
    if !post_draft_paths.is_empty() {
        parse_drafts(&post_draft_paths, &mut posts, &context.posts)?;
    }

    let documents = parse_pages(&page_paths, &context.pages, context.include_drafts)?;

    sort_pages(&mut posts, &context.posts)?;
    generate_posts(&mut posts, &context)?;

    // check if we should create an RSS file and create it!
    if let Some(ref path) = context.posts.rss {
        let path = path.to_path(&context.destination);
        create_rss(
            &path,
            &context.posts,
            &posts,
            context.site.base_url.as_deref(),
        )?;
    }
    // check if we should create an jsonfeed file and create it!
    if let Some(ref path) = context.posts.jsonfeed {
        let path = path.to_path(&context.destination);
        create_jsonfeed(
            &path,
            &context.posts,
            &posts,
            context.site.base_url.as_deref(),
        )?;
    }
    if let Some(ref path) = context.site.sitemap {
        let path = path.to_path(&context.destination);
        create_sitemap(&path, &posts, &documents, context.site.base_url.as_deref())?;
    }

    generate_pages(posts, documents, &context)?;

    // copy all remaining files in the source to the destination
    // compile SASS along the way
    for asset_path in asset_paths {
        context
            .assets
            .process(&asset_path.abs_path, &context.destination, &context.minify)?;
    }

    Ok(())
}

fn generate_collections_var(
    posts_data: &[liquid::model::Value],
    context: &Context,
) -> (liquid::model::KString, liquid::model::Value) {
    let mut posts_variable = context.posts.attributes();
    posts_variable.insert(
        "pages".into(),
        liquid::model::Value::Array(posts_data.to_vec()),
    );
    let global_collection: liquid::Object = vec![(
        context.posts.slug.clone(),
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
    global_collection: (liquid::model::KString, liquid::model::Value),
) -> Result<()> {
    // Everything done with `globals` is terrible for performance.  liquid#95 allows us to
    // improve this.
    let mut globals: liquid::Object = vec![
        (
            "site".into(),
            liquid::model::Value::Object(context.site_attributes.clone()),
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
        let render_context = RenderContext {
            parser: &context.liquid,
            markdown: &context.markdown,
            vimwiki: &context.vimwiki,
            globals: &globals,
            minify: context.minify.clone(),
        };

        doc.render_excerpt(&render_context).with_context(|_| {
            failure::format_err!("Failed to render excerpt for {}", doc.file_path)
        })?;
        doc.render_content(&render_context).with_context(|_| {
            failure::format_err!("Failed to render content for {}", doc.file_path)
        })?;
    }

    // Refresh `page` with the `excerpt` / `content` attribute
    globals.insert(
        "page".into(),
        liquid::model::Value::Object(doc.attributes.clone()),
    );
    let render_context = RenderContext {
        parser: &context.liquid,
        markdown: &context.markdown,
        vimwiki: &context.vimwiki,
        globals: &globals,
        minify: context.minify.clone(),
    };
    let doc_html = doc
        .render(&render_context, &context.layouts)
        .with_context(|_| failure::format_err!("Failed to render for {}", doc.file_path))?;
    files::write_document_file(doc_html, doc.file_path.to_path(&context.destination))?;
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
                generate_collections_var(&posts_data, context),
            )?;
        };
    }
    Ok(())
}

fn generate_posts(posts: &mut [Document], context: &Context) -> Result<()> {
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
            generate_collections_var(&simple_posts_data, context),
        )?;
    }

    Ok(())
}

fn sort_pages(posts: &mut [Document], collection: &Collection) -> Result<()> {
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
    page_paths: &[cobalt_core::SourcePath],
    documents: &mut Vec<Document>,
    collection: &Collection,
) -> Result<()> {
    let dir = &collection.dir;
    let drafts_dir = collection
        .drafts_dir
        .as_deref()
        .expect("Caller checked first");
    for file_path in page_paths {
        // Provide a fake path as if it was not a draft
        let rel_src = file_path
            .rel_path
            .strip_prefix(drafts_dir)
            .expect("file was found under the root");
        let new_path = dir.join(&rel_src);

        let default_front = cobalt_config::Frontmatter {
            is_draft: Some(true),
            ..Default::default()
        }
        .merge(&collection.default);

        let doc = Document::parse(&file_path.abs_path, &new_path, default_front)
            .with_context(|_| failure::format_err!("Failed to parse {}", file_path.rel_path))?;
        documents.push(doc);
    }
    Ok(())
}

fn parse_pages(
    page_paths: &[cobalt_core::SourcePath],
    collection: &Collection,
    include_drafts: bool,
) -> Result<Vec<Document>> {
    let mut documents = vec![];
    for file_path in page_paths {
        let default_front = collection.default.clone();

        let doc = Document::parse(&file_path.abs_path, &file_path.rel_path, default_front)
            .with_context(|_| failure::format_err!("Failed to parse {}", file_path.rel_path))?;
        if !doc.front.is_draft || include_drafts {
            documents.push(doc);
        } else {
            log::trace!("Skipping draft {}", file_path.rel_path);
        }
    }
    Ok(documents)
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

// creates a new RSS file with the contents of the site blog
fn create_rss(
    path: &std::path::Path,
    collection: &Collection,
    documents: &[Document],
    base_url: Option<&str>,
) -> Result<()> {
    debug!("Creating RSS file at {}", path.display());

    let title = &collection.title;
    let description = collection.description.as_deref().unwrap_or("");
    let link = base_url
        .as_ref()
        .ok_or_else(|| failure::err_msg("`base_url` is required for RSS support"))?;

    let items: Result<Vec<rss::Item>> = documents.iter().map(|doc| doc.to_rss(link)).collect();
    let items = items?;

    let channel = rss::ChannelBuilder::default()
        .title(title.as_str().to_owned())
        .link(link.to_owned())
        .description(description.to_owned())
        .items(items)
        .build();

    let rss_string = channel.to_string();
    trace!("RSS data: {}", rss_string);

    // create target directories if any exist
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|_| failure::format_err!("Could not create {}", parent.display()))?;
    }

    let mut rss_file = fs::File::create(&path)?;
    rss_file.write_all(&rss_string.into_bytes())?;
    rss_file.write_all(b"\n")?;

    Ok(())
}

// creates a new jsonfeed file with the contents of the site blog
fn create_jsonfeed(
    path: &std::path::Path,
    collection: &Collection,
    documents: &[Document],
    base_url: Option<&str>,
) -> Result<()> {
    debug!("Creating jsonfeed file at {}", path.display());

    let title = &collection.title;
    let description = collection.description.as_deref().unwrap_or("");
    let link = base_url
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
    files::write_document_file(jsonfeed_string, path)?;

    Ok(())
}

fn create_sitemap(
    path: &path::Path,
    documents: &[Document],
    documents_pages: &[Document],
    base_url: Option<&str>,
) -> Result<()> {
    debug!("Creating sitemap file at {}", path.display());
    let mut buff = Vec::new();
    let writer = SiteMapWriter::new(&mut buff);
    let link = base_url
        .as_ref()
        .ok_or_else(|| failure::err_msg("`base_url` is required for sitemap support"))?;
    let mut urls = writer.start_urlset()?;
    for doc in documents {
        doc.to_sitemap(link, &mut urls)?;
    }

    let link = base_url
        .as_ref()
        .ok_or_else(|| failure::err_msg("`base_url` is required for sitemap support"))?;
    for doc in documents_pages {
        doc.to_sitemap(link, &mut urls)?;
    }
    urls.end()?;

    files::write_document_file(String::from_utf8(buff)?, path)?;

    Ok(())
}

pub fn classify_path<'s>(
    path: &relative_path::RelativePathBuf,
    pages: &'s cobalt_model::Collection,
    posts: &'s cobalt_model::Collection,
    page_extensions: &[liquid::model::KString],
) -> Option<(&'s str, bool)> {
    if ext_contains(page_extensions, path) {
        if path.starts_with(&posts.dir) {
            return Some((posts.slug.as_str(), false));
        }

        if let Some(drafts_dir) = posts.drafts_dir.as_ref() {
            if path.starts_with(drafts_dir) {
                return Some((posts.slug.as_str(), true));
            }
        }

        Some((pages.slug.as_str(), false))
    } else {
        None
    }
}

fn ext_contains(extensions: &[liquid::model::KString], file: &relative_path::RelativePath) -> bool {
    if extensions.is_empty() {
        return true;
    }

    file.extension()
        .map(|ext| extensions.iter().any(|e| e == ext))
        .unwrap_or(false)
}
