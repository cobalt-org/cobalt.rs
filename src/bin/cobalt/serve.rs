use std::fs;
use std::io::prelude::*;
use std::path;
use std::process;
use std::sync::mpsc::channel;
use std::thread;

use clap;
use cobalt;
use cobalt::files;
use hyper;
use hyper::server::{Server, Request, Response};
use hyper::uri::RequestUri;
use notify::{Watcher, RecursiveMode, raw_watcher};

use build;
use error::*;

pub fn watch_command(config: cobalt::Config, matches: &clap::ArgMatches) -> Result<()> {
    build::build(&config)?;

    let source = path::Path::new(&config.source);
    let dest = path::Path::new(&config.dest).to_owned();
    let ignore_dest = {
        let ignore_dest = dest.join("**/*");
        let ignore_dest = ignore_dest
            .to_str()
            .ok_or_else(|| format!("Cannot convert pathname {:?} to UTF-8", dest))?
            .to_owned();
        Some(ignore_dest)
    };
    let port = matches.value_of("port").unwrap().to_string();
    thread::spawn(move || if serve(&dest, &port).is_err() {
                      process::exit(1)
                  });

    let (tx, rx) = channel();
    let mut watcher = raw_watcher(tx).chain_err(|| "Notify error")?;
    watcher
        .watch(&config.source, RecursiveMode::Recursive)
        .chain_err(|| "Notify error")?;
    info!("Watching {:?} for changes", &config.source);

    loop {
        let event = rx.recv().chain_err(|| "Notify error")?;
        let rebuild = if let Some(ref event_path) = event.path {
            // Be as broad as possible in what can cause a rebuild to
            // ensure we don't miss anything (normal file walks will miss
            // `_layouts`, etc).
            let mut page_files = files::FilesBuilder::new(source)?;
            page_files.add_ignore("!.*")?.add_ignore("!_*")?;
            if let Some(ref ignore_dest) = ignore_dest {
                page_files.add_ignore(ignore_dest)?;
            }
            let page_files = page_files.build()?;

            if page_files.includes_file(event_path) {
                trace!("Page changed {:?}", event);
                true
            } else {
                trace!("Ignored file changed {:?}", event);
                false
            }
        } else {
            trace!("Assuming change {:?} is relevant", event);
            true
        };
        if rebuild {
            build::build(&config)?;
        }
    }
}

pub fn serve_command(config: cobalt::Config, matches: &clap::ArgMatches) -> Result<()> {
    build::build(&config)?;
    let port = matches.value_of("port").unwrap().to_string();
    let dest = path::Path::new(&config.dest);
    serve(dest, &port)?;

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

fn serve(dest: &path::Path, port: &str) -> Result<()> {
    info!("Serving {:?} through static file server", dest);

    let ip = format!("127.0.0.1:{}", port);
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
    if let Err(e) = http_server.handle(move |req: Request, res: Response| if let Err(e) =
        static_file_handler(&dest_clone, req, res) {
                                           error!("{}", e);
                                           process::exit(1);
                                       }) {
        error!("{}", e);
        return Err(e.into());
    };

    Ok(())
}
