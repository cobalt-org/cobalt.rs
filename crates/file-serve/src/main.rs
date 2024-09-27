fn main() {
    let path = match std::env::current_dir() {
        Ok(path) => path,
        Err(err) => {
            eprintln!("Cannot serve CWD: {err}");
            std::process::exit(1);
        }
    };
    let server = file_serve::Server::new(&path);

    println!("Serving {}", path.display());
    println!("See http://{}", server.addr());
    println!("Hit CTRL-C to stop");

    server.serve().unwrap();
}
