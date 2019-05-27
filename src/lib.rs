use std::{io, fs};
use std::io::prelude::*;
use std::io::ErrorKind;
use std::net::TcpStream;

use bufstream::BufStream;
use chrono::prelude::*;
use http::StatusCode;


struct Request {
    http_version: String,
    method: String,
    path: String,
    time: DateTime<Local>,
}

enum ContentType {
    CSS,
    GIF,
    HTML,
    JPEG,
    PNG,
    SVG,
    TEXT,
    XML,
}

impl ContentType {
    fn from_file_ext(ext: &str) -> ContentType {
        match ext {
            "css" => ContentType::CSS,
            "gif" => ContentType::GIF,
            "htm" => ContentType::HTML,
            "html" => ContentType::HTML,
            "jpeg" => ContentType::JPEG,
            "jpg" => ContentType::JPEG,
            "png" => ContentType::PNG,
            "svg" => ContentType::SVG,
            "txt" => ContentType::TEXT,
            "xml" => ContentType::XML,
            _ => ContentType::TEXT,
        }
    }

    fn value(&self) -> &str {
        match *self {
            ContentType::CSS => "text/css",
            ContentType::GIF => "image/gif",
            ContentType::HTML => "text/html",
            ContentType::JPEG => "image/jpeg",
            ContentType::PNG => "image/png",
            ContentType::SVG => "image/svg+xml",
            ContentType::TEXT => "text/plain",
            ContentType::XML => "application/xml",
        }
    }
}

struct ResponseHeaders {
    content_type: Option<ContentType>,
}

impl ResponseHeaders {
    fn new() -> ResponseHeaders {
        ResponseHeaders {
            content_type: None,
        }
    }
}

struct Response {
    body: Option<Vec<u8>>,
    headers: ResponseHeaders,
    status: StatusCode,
}

impl Response {
    fn new() -> Response {
        Response {
            body: None,
            headers: ResponseHeaders::new(),
            status: StatusCode::OK,
        }
    }
}

pub fn handle_client(stream: TcpStream, static_root: &str) -> io::Result<()> {
    let mut buf = BufStream::new(stream);
    let mut request_line = String::new();

    // Get only the first line of the request, since this
    // is a static HTTP 1.0 server.
    buf.read_line(&mut request_line)?;
    let response = match parse_request(&mut request_line) {
        Ok(request) => {
            let response = build_response(&request, static_root);
            log_request(&request, &response);
            response
        },
        Err(()) => create_bad_request_response(),
    };

    let formatted = format_response(response);
    buf.write_all(&formatted)?;

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

fn build_response(request: &Request, static_root: &str) -> Response {
    let mut response = Response::new();
    if request.method != "GET" {
        response.status = StatusCode::METHOD_NOT_ALLOWED;
    } else {
        add_file_to_response(&request.path, &mut response, static_root);
    }

    response
}

fn add_file_to_response(path: &String, response: &mut Response, static_root: &str) {
    let path = format!("{}{}", static_root, path);
    let contents = fs::read(&path);
    match contents {
        Ok(contents) => {
            response.body = Some(contents);
            let ext = path.split(".").last().unwrap_or("");
            response.headers.content_type = Some(ContentType::from_file_ext(ext));
        },
        Err(e) => {
            response.status = match e.kind() {
                ErrorKind::NotFound => StatusCode::NOT_FOUND,
                ErrorKind::PermissionDenied => StatusCode::FORBIDDEN,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            }
        }
    }
}

fn log_request(request: &Request, response: &Response) {
    println!(
        "[{}] \"{} {} {}\" {}",
        request.time,
        request.method,
        request.path,
        request.http_version,
        response.status.as_u16(),
    );
}

fn create_bad_request_response() -> Response {
    let mut response = Response::new();
    response.status = StatusCode::BAD_REQUEST;

    response
}

fn format_response(response: Response) -> Vec<u8> {
    let mut result;
    let status_reason = match response.status.canonical_reason() {
        Some(reason) => reason,
        None => "",
    };
    result = format!(
        "HTTP/1.0 {} {}\n",
        response.status.as_str(),
        status_reason,
    );
    result = format!("{}Allow: GET\n", result);

    match response.headers.content_type {
        Some(content_type) => {
            result = format!(
                "{}Content-type: {}\n", result, content_type.value());
        },
        _ => (),
    }

    let mut bytes = result.as_bytes().to_vec();

    match response.body {
        Some(mut body) => {
            bytes.append(&mut "\n".as_bytes().to_vec());
            bytes.append(&mut body);
        },
        _ => (),
    }

    bytes
}
