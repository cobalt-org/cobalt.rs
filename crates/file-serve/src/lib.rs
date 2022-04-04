//! > An HTTP Static File Server
//!
//! `file-serve` focuses on augmenting development of your site.  It prioritizes
//! small size and compile times over speed, scalability, or security.
//!
//! # Example
//!
//! ```rust,no_run
//! let path = std::env::current_dir().unwrap();
//! let server = file_serve::Server::new(&path);
//!
//! println!("Serving {}", path.display());
//! println!("See http://{}", server.addr());
//! println!("Hit CTRL-C to stop");
//!
//! server.serve().unwrap();
//! ```

use std::str::FromStr;

/// Custom server settings
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ServerBuilder {
    source: std::path::PathBuf,
    hostname: Option<String>,
    port: Option<u16>,
}

impl ServerBuilder {
    pub fn new(source: impl Into<std::path::PathBuf>) -> Self {
        Self {
            source: source.into(),
            hostname: None,
            port: None,
        }
    }

    /// Override the hostname
    pub fn hostname(&mut self, hostname: impl Into<String>) -> &mut Self {
        self.hostname = Some(hostname.into());
        self
    }

    /// Override the port
    ///
    /// By default, the first available port is selected.
    pub fn port(&mut self, port: u16) -> &mut Self {
        self.port = Some(port);
        self
    }

    /// Create a server
    ///
    /// This is needed for accessing the dynamically assigned pot
    pub fn build(&self) -> Server {
        let source = self.source.clone();
        let hostname = self.hostname.as_deref().unwrap_or("localhost");
        let port = self
            .port
            .or_else(|| get_available_port(hostname))
            // Just have `serve` error out
            .unwrap_or(3000);

        Server {
            source,
            addr: format!("{}:{}", hostname, port),
        }
    }

    /// Start the webserver
    pub fn serve(&self) -> Result<std::convert::Infallible, Error> {
        self.build().serve()
    }
}

#[derive(Debug)]
pub struct Server {
    source: std::path::PathBuf,
    addr: String,
}

impl Server {
    /// Serve on first available port on localhost
    pub fn new(source: impl Into<std::path::PathBuf>) -> Self {
        ServerBuilder::new(source).build()
    }

    /// The location being served
    pub fn source(&self) -> &std::path::Path {
        self.source.as_path()
    }

    /// The address the server is available at
    ///
    /// This is useful for telling users how to access the served up files since the port is
    /// dynamically assigned by default.
    pub fn addr(&self) -> &str {
        self.addr.as_str()
    }

    /// Start the webserver
    pub fn serve(&self) -> Result<std::convert::Infallible, Error> {
        // attempts to create a server
        let server = tiny_http::Server::http(self.addr()).map_err(Error::new)?;

        for request in server.incoming_requests() {
            if let Err(e) = static_file_handler(self.source(), request) {
                log::error!("{}", e);
            }
        }
        unreachable!("`incoming_requests` never stops")
    }
}

/// Serve Error
#[derive(Debug)]
pub struct Error {
    message: String,
}

impl Error {
    fn new(message: impl ToString) -> Self {
        Self {
            message: message.to_string(),
        }
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.message.fmt(fmt)
    }
}

impl std::error::Error for Error {}

fn static_file_handler(dest: &std::path::Path, req: tiny_http::Request) -> Result<(), Error> {
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
        let file = std::fs::File::open(&serve_path).map_err(Error::new)?;
        let mut response = tiny_http::Response::from_file(file);
        if let Some(mime) = mime_guess::MimeGuess::from_path(&serve_path).first_raw() {
            let content_type = format!("Content-Type:{}", mime);
            let content_type =
                tiny_http::Header::from_str(&content_type).expect("formatted correctly");
            response.add_header(content_type);
        }
        req.respond(response).map_err(Error::new)?;
    } else {
        // write a simple body for the 404 page
        req.respond(
            tiny_http::Response::from_string("<h1> <center> 404: Page not found </center> </h1>")
                .with_status_code(404)
                .with_header(
                    tiny_http::Header::from_str("Content-Type: text/html")
                        .expect("formatted correctly"),
                ),
        )
        .map_err(Error::new)?;
    }

    Ok(())
}

fn get_available_port(host: &str) -> Option<u16> {
    // Start after "well-known" ports (0–1023) as they require superuser
    // privileges on UNIX-like operating systems.
    (1024..9000).find(|port| port_is_available(host, *port))
}

fn port_is_available(host: &str, port: u16) -> bool {
    std::net::TcpListener::bind((host, port)).is_ok()
}
