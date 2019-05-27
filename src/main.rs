use std::io;
use std::net::TcpListener;

use rust_http_server;

extern crate chrono;


fn main() -> io::Result<()> {
    println!("Starting server...");

    let listener = TcpListener::bind("127.0.0.1:8001")?;
    const STATIC_ROOT: &str = "/static_root";

    println!("Server started!");

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                match rust_http_server::handle_client(stream, &STATIC_ROOT) {
                    Err(e) => eprintln!("Error handling client: {}", e),
                    _ => (),
                }
            },
            Err(e) => eprintln!("Connection failed: {}", e),
        }
    }

    Ok(())
}
