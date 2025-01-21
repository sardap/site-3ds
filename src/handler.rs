use chrono::prelude::*;
use std::net::{Shutdown, TcpListener};
use std::time::Duration;

use crate::api;
use crate::database::Database;
use crate::http_utils::{Request, Response, ResponseBody, content_types};

include!(concat!(env!("OUT_DIR"), "/dist.rs"));

pub struct Handler {
    server: TcpListener,
}

impl Handler {
    pub fn new() -> Self {
        let server = TcpListener::bind("0.0.0.0:8081").unwrap();
        server.set_nonblocking(true).unwrap();
        Self { server }
    }

    pub fn step(&mut self, db: &mut Database) {
        match self.server.accept() {
            Ok((mut stream, _socket_addr)) => {
                let (request, response) = if let Some(request) = Request::from(&stream) {
                    let response = self.route(&request, db);
                    (request, response)
                } else {
                    (bad_request(), INTERNAL_SERVER_ERROR)
                };
                println!(
                    "{} {} {} {}\n",
                    Utc::now().format("%Y-%m-%d %H:%M:%S"),
                    request.method,
                    request.path,
                    response.status
                );
                response.send(&mut stream);

                // Shutdown the stream (depending on the web browser used to view the page, this might cause some issues).
                match stream.shutdown(Shutdown::Both) {
                    Ok(_) => {}
                    Err(e) => {
                        println!("Error shutting down stream: {e}");
                    }
                }
            }
            Err(e) => match e.kind() {
                // If the TCP socket would block execution, just try again.
                std::io::ErrorKind::WouldBlock => {}
                _ => {
                    println!("Error accepting connection: {e}");
                    std::thread::sleep(Duration::from_secs(2));
                }
            },
        }
    }

    fn route<'a>(&self, request: &Request, db: &mut Database) -> Response<'a> {
        if request.path.starts_with("/api") {
            if let Some(value) = api::route(request, db) {
                return value;
            }
        }

        for serve_request in SERVE_REQUESTS.iter() {
            if serve_request.method == request.method && serve_request.path == request.path {
                let mut response = Response::new();
                response.content_type = serve_request.content_type;
                response.body = ResponseBody::Lifetime(serve_request.body);
                return response;
            }
        }

        if request.method == "GET" && request.path == "/" {
            let mut response = Response::new();
            response.content_type = REQUEST_INDEX_HTML.content_type;
            response.body = ResponseBody::Lifetime(REQUEST_INDEX_HTML.body);
            return response;
        }

        let mut response = Response::new();
        response.status = 404;
        response.content_type = content_types::HTML;
        response.body =
            ResponseBody::Lifetime("<html><body><h1>404 Not Found</h1></body></html>".as_bytes());
        response
    }
}

fn bad_request() -> Request {
    Request {
        method: String::from("???"),
        path: String::from("???"),
        version: 1.1,
        headers: vec![],
        body: String::from("???"),
    }
}

const INTERNAL_SERVER_ERROR: Response = Response {
    version: 1.1,
    status: 500,
    content_type: content_types::HTML,
    headers: vec![],
    body: ResponseBody::Lifetime(
        "<html><body><h1>500 Internal Server Error</h1></body></html>".as_bytes(),
    ),
};
