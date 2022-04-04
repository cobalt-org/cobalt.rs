fn main() {
    let path = std::env::current_dir().unwrap();
    let server = file_serve::Server::new(&path);

    println!("Serving {}", path.display());
    println!("See http://{}", server.addr());
    println!("Hit CTRL-C to stop");

    server.serve().unwrap();
}
