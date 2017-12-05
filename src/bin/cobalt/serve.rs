use std::fs;
use std::io::prelude::*;
use std::path;
use std::process;
use std::sync::mpsc::channel;
use std::thread;

use clap;
use cobalt::cobalt_model::files;
use hyper;
use hyper::server::{Server, Request, Response};
use hyper::uri::RequestUri;
use notify::{Watcher, RecursiveMode, raw_watcher};

use args;
use build;
use error::*;

pub fn watch_command_args() -> clap::App<'static, 'static> {
    clap::SubCommand::with_name("watch")
        .about("build, serve, and watch the project at the source dir")
        .args(&args::get_config_args())
        .arg(clap::Arg::with_name("port")
                 .short("P")
                 .long("port")
                 .value_name("INT")
                 .help("Port to serve from")
                 .default_value("3000")
                 .takes_value(true))
}

pub fn watch_command(matches: &clap::ArgMatches) -> Result<()> {
    let config = args::get_config(matches)?;
    let config = config.build()?;

    build::build(&config)?;

    // canonicalize is to ensure there is no question that `watcher`s paths come back safe for
    // Files::includes_file
    let source = path::Path::new(&config.source)
        .canonicalize()
        .chain_err(|| "Failed in processing source")?;
    let dest = path::Path::new(&config.destination).to_owned();

    // Be as broad as possible in what can cause a rebuild to
    // ensure we don't miss anything (normal file walks will miss
    // `_layouts`, etc).
    let mut site_files = files::FilesBuilder::new(&source)?;
    site_files.ignore_hidden(false)?;
    for line in &config.ignore {
        site_files.add_ignore(line.as_str())?;
    }
    let site_files = site_files.build()?;

    let port = matches.value_of("port").unwrap().to_string();
    thread::spawn(move || if serve(&dest, &port).is_err() {
                      process::exit(1)
                  });

    let (tx, rx) = channel();
    let mut watcher = raw_watcher(tx).chain_err(|| "Notify error")?;
    watcher
        .watch(&source, RecursiveMode::Recursive)
        .chain_err(|| "Notify error")?;
    info!("Watching {:?} for changes", &config.source);

    loop {
        let event = rx.recv().chain_err(|| "Notify error")?;
        let rebuild = if let Some(ref event_path) = event.path {
            if site_files.includes_file(event_path) {
                debug!("Page changed {:?}", event);
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

pub fn serve_command_args() -> clap::App<'static, 'static> {
    clap::SubCommand::with_name("serve")
        .about("build and serve the cobalt project at the source dir")
        .args(&args::get_config_args())
        .arg(clap::Arg::with_name("port")
                 .short("P")
                 .long("port")
                 .value_name("INT")
                 .help("Port to serve from")
                 .default_value("3000")
                 .takes_value(true))
}

pub fn serve_command(matches: &clap::ArgMatches) -> Result<()> {
    let config = args::get_config(matches)?;
    let config = config.build()?;

    build::build(&config)?;
    let port = matches.value_of("port").unwrap().to_string();
    let dest = path::Path::new(&config.destination);
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
