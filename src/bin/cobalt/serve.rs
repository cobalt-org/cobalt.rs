use std::fs;
use std::io::prelude::*;
use std::path;
use std::process;
use std::sync::mpsc::channel;
use std::thread;
use std::time;

use clap;
use cobalt::cobalt_model;
use error_chain::ChainedError;
use hyper;
use hyper::server::{Request, Response, Server};
use hyper::uri::RequestUri;
use notify;
use notify::Watcher;

use args;
use build;
use error::*;

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

fn static_file_handler(dest: &path::Path, req: Request, mut res: Response) -> Result<()> {
    // grab the requested path
    let mut req_path = match req.uri {
        RequestUri::AbsolutePath(p) => p,
        _ => {
            // return a 400 and exit from this request
            *res.status_mut() = hyper::status::StatusCode::BadRequest;
            let body = b"<h1> <center> 400: Bad request </center> </h1>";
            res.send(body)?;
            return Ok(());
        }
    };

    // strip off any querystrings so path.is_file() matches
    // and doesn't stick index.html on the end of the path
    // (querystrings often used for cachebusting)
    if let Some(position) = req_path.rfind('?') {
        req_path.truncate(position);
    }

    // find the path of the file in the local system
    // (this gets rid of the '/' in `p`, so the `join()` will not replace the
    // path)
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
        let mut file = fs::File::open(serve_path)?;

        // buffer to store the file
        let mut buffer: Vec<u8> = vec![];

        file.read_to_end(&mut buffer)?;

        res.send(&buffer)?;
    } else {
        // return a 404 status
        *res.status_mut() = hyper::status::StatusCode::NotFound;

        // write a simple body for the 404 page
        let body = b"<h1> <center> 404: Page not found </center> </h1>";

        res.send(body)?;
    }

    Ok(())
}

fn serve(dest: &path::Path, ip: &str) -> Result<()> {
    info!("Serving {:?} through static file server", dest);

    info!("Server Listening on {}", &ip);
    info!("Ctrl-c to stop the server");

    // attempts to create a server
    let http_server = match Server::http(&*ip) {
        Ok(server) => server,
        Err(e) => {
            error!("{}", e);
            return Err(e.into());
        }
    };

    // need a clone because of closure's lifetime
    let dest_clone = dest.to_owned();

    // bind the handle function and start serving
    if let Err(e) = http_server.handle(move |req: Request, res: Response| {
        if let Err(e) = static_file_handler(&dest_clone, req, res) {
            error!("{}", e);
            process::exit(1);
        }
    }) {
        error!("{}", e);
        return Err(e.into());
    };

    Ok(())
}

fn watch(config: &cobalt_model::Config) -> Result<()> {
    // canonicalize is to ensure there is no question that `watcher`s paths come back safe for
    // Files::includes_file
    let source = path::Path::new(&config.source)
        .canonicalize()
        .chain_err(|| "Failed in processing source")?;

    let (tx, rx) = channel();
    let mut watcher =
        notify::watcher(tx, time::Duration::from_secs(1)).chain_err(|| "Notify error")?;
    watcher
        .watch(&source, notify::RecursiveMode::Recursive)
        .chain_err(|| "Notify error")?;
    info!("Watching {:?} for changes", &config.source);

    loop {
        let event = rx.recv().chain_err(|| "Notify error")?;
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
                error!("build failed\n{}", fail.display_chain());
            }
        }
    }
}
