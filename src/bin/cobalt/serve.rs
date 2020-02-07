use std::fs;
use std::io::prelude::*;
use std::path;
use std::process;
use std::sync::mpsc::channel;
use std::thread;
use std::time;

use actix_files;
use actix_web::middleware::errhandlers::{ErrorHandlerResponse, ErrorHandlers};
use actix_web::{dev, http, App, HttpResponse, HttpServer};
use clap;
use cobalt::cobalt_model;
use failure::ResultExt;
use notify;
use notify::Watcher;

use crate::args;
use crate::build;
use crate::error::*;

pub fn serve_command_args() -> clap::App<'static, 'static> {
    clap::SubCommand::with_name("serve")
        .about("build, serve, and watch the project at the source dir")
        .args(&args::get_config_args())
        .arg(
            clap::Arg::with_name("port")
                .short("P")
                .long("port")
                .value_name("INT")
                .help("Port to serve from")
                .default_value("3000")
                .takes_value(true),
        )
        .arg(
            clap::Arg::with_name("host")
                .long("host")
                .value_name("host-name/IP")
                .help("Host to serve from")
                .default_value("localhost")
                .takes_value(true),
        )
        .arg(
            clap::Arg::with_name("no-watch")
                .long("no-watch")
                .help("Disable rebuilding on change")
                .conflicts_with("drafts")
                .takes_value(false),
        )
}

pub fn serve_command(matches: &clap::ArgMatches) -> Result<()> {
    let host = matches.value_of("host").unwrap().to_string();
    let port = matches.value_of("port").unwrap().to_string();
    let ip = format!("{}:{}", host, port);

    let mut config = args::get_config(matches)?;
    debug!("Overriding config `site.base_url` with `{}`", ip);
    config.site.base_url = Some(format!("http://{}", ip));
    let config = config.build()?;
    let dest = path::Path::new(&config.destination).to_owned();

    build::build(config.clone())?;

    if matches.is_present("no-watch") {
        serve(&dest, &ip)?;
    } else {
        thread::spawn(move || {
            if serve(&dest, &ip).is_err() {
                process::exit(1)
            }
        });

        watch(&config)?;
    }
    Ok(())
}

fn watch(config: &cobalt_model::Config) -> Result<()> {
    // canonicalize is to ensure there is no question that `watcher`s paths come back safe for
    // Files::includes_file
    let source = path::Path::new(&config.source)
        .canonicalize()
        .with_context(|_| failure::err_msg("Failed in processing source"))?;

    let (tx, rx) = channel();
    let mut watcher = notify::watcher(tx, time::Duration::from_secs(1))
        .with_context(|_| failure::err_msg("Notify error"))?;
    watcher
        .watch(&source, notify::RecursiveMode::Recursive)
        .with_context(|_| failure::err_msg("Notify error"))?;
    info!("Watching {:?} for changes", &config.source);

    loop {
        let event = rx
            .recv()
            .with_context(|_| failure::err_msg("Notify error"))?;
        let event_path = match event {
            notify::DebouncedEvent::Create(ref path)
            | notify::DebouncedEvent::NoticeWrite(ref path)
            | notify::DebouncedEvent::Write(ref path)
            | notify::DebouncedEvent::NoticeRemove(ref path)
            | notify::DebouncedEvent::Remove(ref path) => Some(path),
            notify::DebouncedEvent::Rename(_, ref to) => Some(to),
            _ => None,
        };
        let rebuild = if let Some(event_path) = event_path {
            // Be as broad as possible in what can cause a rebuild to
            // ensure we don't miss anything (normal file walks will miss
            // `_layouts`, etc).
            if event_path.starts_with(&config.destination) {
                trace!("Ignored file changed {:?}", event);
                false
            } else {
                debug!("Page changed {:?}", event);
                true
            }
        } else {
            trace!("Assuming change {:?} is relevant", event);
            true
        };
        if rebuild {
            let result = build::build(config.clone());
            if let Err(fail) = result {
                let fail: exitfailure::ExitFailure = fail.into();
                error!("build failed\n{:?}", fail);
            }
        }
    }
}

fn serve(dest: &path::Path, ip: &str) -> Result<()> {
    let dest = dest.to_owned();
    info!("Serving {:?} through static file server", dest);

    info!("Server Listening on http://{}", &ip);
    info!("Ctrl-c to stop the server");

    let s = HttpServer::new(move || {
        let error_handlers = ErrorHandlers::new().handler(http::StatusCode::NOT_FOUND, not_found);

        App::new()
            .data(ErrorFilePaths {
                not_found: dest.join("404.html"),
            })
            .wrap(error_handlers)
            // Start a webserver that serves the `output_dir` directory
            .service(actix_files::Files::new("/", &dest).index_file("index.html"))
    })
    .bind(&ip)
    .expect("Can't start the webserver")
    .shutdown_timeout(20);
    s.run()?;

    Ok(())
}

struct ErrorFilePaths {
    not_found: path::PathBuf,
}

fn not_found<B>(
    res: dev::ServiceResponse<B>,
) -> std::result::Result<ErrorHandlerResponse<B>, actix_web::Error> {
    let buf: Vec<u8> = {
        let error_files: &ErrorFilePaths = res.request().app_data().unwrap();

        let mut fh = fs::File::open(&error_files.not_found)?;
        let mut buf: Vec<u8> = vec![];
        let _ = fh.read_to_end(&mut buf)?;
        buf
    };

    let new_resp = HttpResponse::build(http::StatusCode::NOT_FOUND)
        .header(
            http::header::CONTENT_TYPE,
            http::header::HeaderValue::from_static("text/html"),
        )
        .body(buf);

    Ok(ErrorHandlerResponse::Response(
        res.into_response(new_resp.into_body()),
    ))
}
