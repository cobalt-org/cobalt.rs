use std::path;
use std::process;
use std::sync::mpsc::channel;
use std::thread;
use std::time;

use cobalt::cobalt_model;
use failure::ResultExt;
use notify::Watcher;

use crate::args;
use crate::build;
use crate::error::*;

/// Build, serve, and watch the project at the source dir
#[derive(Clone, Debug, PartialEq, Eq, clap::Args)]
pub struct ServeArgs {
    /// Open a browser
    #[clap(long)]
    pub open: bool,

    /// Host to serve from
    #[clap(long, value_name = "HOSTNAME_OR_IP", default_value = "localhost")]
    pub host: String,

    /// Port to serve from
    #[clap(short = 'P', long, value_name = "NUM")]
    pub port: Option<u16>,

    /// Disable rebuilding on change
    #[clap(long)]
    pub no_watch: bool,

    #[clap(flatten, help_heading = "CONFIG")]
    pub config: args::ConfigArgs,
}

impl ServeArgs {
    pub fn run(&self) -> Result<()> {
        let dest = tempfile::tempdir()?;

        let mut server = file_serve::ServerBuilder::new(dest.path());
        server.hostname(&self.host);
        if let Some(port) = self.port {
            server.port(port);
        }
        let server = server.build();

        let mut config = self.config.load_config()?;
        debug!("Overriding config `site.base_url` with `{}`", server.addr());
        config.site.base_url = Some(format!("http://{}", server.addr()).into());
        let mut config = cobalt::cobalt_model::Config::from_config(config)?;
        debug!(
            "Overriding config `destination` with `{}`",
            dest.path().display()
        );
        config.destination = dest.path().to_owned();

        build::build(config.clone())?;

        if self.open {
            let url = format!("http://{}", server.addr());
            open_browser(url)?;
        }

        if self.no_watch {
            serve(&server)?;

            dest.close()?;
        } else {
            info!("Watching {:?} for changes", &config.source);
            thread::spawn(move || {
                let e = serve(&server);
                if let Some(e) = e.err() {
                    error!("{}", e);
                }
                process::exit(1)
            });

            watch(&config)?;
        }

        Ok(())
    }
}

fn serve(server: &file_serve::Server) -> Result<()> {
    info!(
        "Serving {} through static file server",
        server.source().display()
    );
    info!("Server Listening on http://{}", server.addr());
    info!("Ctrl-c to stop the server");

    server
        .serve()
        .map_err(|e| Error::from_boxed_compat(Box::new(e)))
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
    let source = dunce::canonicalize(path::Path::new(&config.source))
        .with_context(|_| failure::err_msg("Failed in processing source"))?;

    // Also canonicalize the destination folder. In particular for Windows, notify-rs
    // generates the absolute path by prepending the above source path.
    // On Windows canonicalize() adds a \\?\ to the start of the path.
    let destination = dunce::canonicalize(&config.destination)
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
