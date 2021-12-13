use std::fs;
use std::path;
use std::process;
use std::str::FromStr;
use std::sync::mpsc::channel;
use std::thread;
use std::time;

use cobalt::cobalt_model;
use failure::ResultExt;
use notify::Watcher;
use tiny_http::{Request, Response, Server};

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
        .arg(
            clap::Arg::with_name("open")
                .long("open")
                .help("Open in browser")
                .takes_value(false),
        )
}

pub fn serve_command(matches: &clap::ArgMatches<'_>) -> Result<()> {
    let host = matches.value_of("host").unwrap().to_string();
    let port = matches.value_of("port").unwrap().to_string();
    let ip = format!("{}:{}", host, port);
    let open_in_browser = matches.is_present("open");
    let url = format!("http://{}", ip);

    let mut config = args::get_config(matches)?;
    debug!("Overriding config `site.base_url` with `{}`", ip);
    config.site.base_url = Some(format!("http://{}", ip).into());
    let config = cobalt::cobalt_model::Config::from_config(config)?;
    let dest = path::Path::new(&config.destination).to_owned();

    build::build(config.clone())?;

    if open_in_browser {
        open_browser(url)?;
    }

    if matches.is_present("no-watch") {
        serve(&dest, &ip)?;
    } else {
        info!("Watching {:?} for changes", &config.source);
        thread::spawn(move || {
            let e = serve(&dest, &ip);
            if let Some(e) = e.err() {
                error!("{}", e);
            }
            process::exit(1)
        });

        watch(&config)?;
    }
    Ok(())
}

fn static_file_handler(dest: &path::Path, req: Request) -> Result<()> {
    // grab the requested path
    let mut req_path = req.url().to_string();

    // strip off any querystrings so path.is_file() matches and doesn't stick index.html on the end
    // of the path (querystrings often used for cachebusting)
    if let Some(position) = req_path.rfind('?') {
        req_path.truncate(position);
    }

    // find the path of the file in the local system
    // (this gets rid of the '/' in `p`, so the `join()` will not replace the path)
    let path = dest.to_path_buf().join(&req_path[1..]);

    let serve_path = if path.is_file() {
        // try to point the serve path to `path` if it corresponds to a file
        path
    } else {
        // try to point the serve path into a "index.html" file in the requested
        // path
        path.join("index.html")
    };

    // if the request points to a file and it exists, read and serve it
    if serve_path.exists() {
        let file = fs::File::open(&serve_path)?;
        let mut response = Response::from_file(file);
        if let Some(mime) = mime_guess::MimeGuess::from_path(&serve_path).first_raw() {
            let content_type = format!("Content-Type:{}", mime);
            let content_type =
                tiny_http::Header::from_str(&content_type).expect("formatted correctly");
            response.add_header(content_type);
        }
        req.respond(response)?;
    } else {
        // write a simple body for the 404 page
        req.respond(
            Response::from_string("<h1> <center> 404: Page not found </center> </h1>")
                .with_status_code(404)
                .with_header(
                    tiny_http::Header::from_str("Content-Type: text/html")
                        .expect("formatted correctly"),
                ),
        )?;
    }

    Ok(())
}

fn serve(dest: &path::Path, ip: &str) -> Result<()> {
    info!("Serving {:?} through static file server", dest);
    info!("Server Listening on http://{}", &ip);
    info!("Ctrl-c to stop the server");

    // attempts to create a server
    let server = Server::http(ip).map_err(Error::from_boxed_compat)?;

    for request in server.incoming_requests() {
        if let Err(e) = static_file_handler(dest, request) {
            error!("{}", e);
        }
    }
    Ok(())
}

fn open_browser(url: String) -> Result<()> {
    match open::that(url) {
        Ok(()) => info!("Please check your browser!"),
        Err(why) => eprintln!("Failure to execute command: {}", why),
    }
    Ok(())
}

fn watch(config: &cobalt_model::Config) -> Result<()> {
    // canonicalize is to ensure there is no question that `watcher`s paths come back safe for
    // Files::includes_file
    let source = path::Path::new(&config.source)
        .canonicalize()
        .with_context(|_| failure::err_msg("Failed in processing source"))?;

    // Also canonicalize the destination folder. In particular for Windows, notify-rs
    // generates the absolute path by prepending the above source path.
    // On Windows canonicalize() adds a \\?\ to the start of the path.
    let destination = config
        .destination
        .canonicalize()
        .with_context(|_| failure::err_msg("Failed to canonicalize destination folder"))?;

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
            if event_path.starts_with(&destination) {
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
