use std::io;
use std::io::prelude::*;
use std::net::TcpStream;

use bufstream::BufStream;
use chrono::prelude::*;


struct Request {
    http_version: String,
    method: String,
    path: String,
    time: DateTime<Local>,
}

pub fn handle_client(stream: TcpStream) -> io::Result<()> {
    let mut buf = BufStream::new(stream);
    let mut request_line = String::new();

    // Get only the first line of the request, since this
    // is a static HTTP 1.0 server.
    buf.read_line(&mut request_line)?;
    match parse_request(&mut request_line) {
        Ok(request) => {
            log_request(&request);
        },
        Err(()) => {
            println!("Bad request: {}", &request_line);
        },
    }

    Ok(())
}

fn parse_request(request: &mut String) -> Result<Request, ()> {
    let mut parts = request.split(" ");
    let method = match parts.next() {
        Some(method) => method.trim().to_string(),
        None => return Err(()),
    };
    let path = match parts.next() {
        Some(path) => path.trim().to_string(),
        None => return Err(()),
    };
    let http_version = match parts.next() {
        Some(version) => version.trim().to_string(),
        None => return Err(()),
    };
    let time = Local::now();

    Ok( Request {
        http_version: http_version,
        method: method,
        path: path,
        time: time
    } )
}

fn log_request(request: &Request) {
    println!(
        "[{}] \"{} {} {}\"",
        request.time,
        request.method,
        request.path,
        request.http_version,
    );
}
