use std::path;
use std::process;
use std::sync::mpsc::channel;
use std::thread;

use anyhow::Context as _;
use cobalt::cobalt_model;
use notify::Watcher as _;

use crate::args;
use crate::build;
use crate::error::*;

/// Build, serve, and watch the project at the source dir
#[derive(Clone, Debug, PartialEq, Eq, clap::Args)]
pub struct ServeArgs {
    /// Open a browser
    #[arg(long)]
    pub open: bool,

    /// Host to serve from
    #[arg(long, value_name = "HOSTNAME_OR_IP", default_value = "localhost")]
    pub host: String,

    /// Port to serve from
    #[arg(short = 'P', long, value_name = "NUM")]
    pub port: Option<u16>,

    /// Disable rebuilding on change
    #[arg(long)]
    pub no_watch: bool,

    #[command(flatten, next_help_heading = "Config")]
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

    Ok(server.serve()?)
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
        .with_context(|| anyhow::format_err!("Failed in processing source"))?;

    // Also canonicalize the destination folder. In particular for Windows, notify-rs
    // generates the absolute path by prepending the above source path.
    // On Windows canonicalize() adds a \\?\ to the start of the path.
    let destination = dunce::canonicalize(&config.destination)
        .with_context(|| anyhow::format_err!("Failed to canonicalize destination folder"))?;

    let (tx, rx) = channel();
    let mut watcher =
        notify::recommended_watcher(tx).with_context(|| anyhow::format_err!("Notify error"))?;
    watcher
        .watch(&source, notify::RecursiveMode::Recursive)
        .with_context(|| anyhow::format_err!("Notify error"))?;
    info!("Watching {:?} for changes", &config.source);

    for event in rx {
        let event = event.with_context(|| anyhow::format_err!("Notify error"))?;
        let event_paths = match event.kind {
            notify::EventKind::Create(_)
            | notify::EventKind::Modify(_)
            | notify::EventKind::Remove(_) => {
                log::trace!("Noticed {:?} for {:#?}", event.kind, event.paths);
                &event.paths
            }
            _ => {
                continue;
            }
        };
        let rebuild = event_paths.iter().any(|event_path| {
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
        });
        if rebuild {
            let result = build::build(config.clone());
            if let Err(fail) = result {
                error!("build failed\n{:?}", fail);
            }
        }
    }

    Ok(())
}
