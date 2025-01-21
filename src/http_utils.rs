use std::{
    io::{self, Read, Write},
    net::TcpStream,
};

fn safe_send(stream: &mut TcpStream, data: &[u8]) {
    let mut offset = 0;
    loop {
        let to_send = &data[offset..data.len()];
        let write = match stream.write(&to_send) {
            Ok(write) => write,
            Err(e) => {
                if e.kind() == io::ErrorKind::WouldBlock {
                    continue;
                } else {
                    println!("Error writing to stream: {e}");
                    break;
                }
            }
        };
        offset += write;
        if offset >= data.len() {
            break;
        }
    }
}

#[derive(Clone, Debug)]
pub enum ResponseBody<'a> {
    Lifetime(&'a [u8]),
    Owned(Vec<u8>),
}

impl ResponseBody<'_> {
    pub fn len(&self) -> usize {
        match self {
            ResponseBody::Lifetime(data) => data.len(),
            ResponseBody::Owned(data) => data.len(),
        }
    }

    pub fn as_slice(&self) -> &[u8] {
        match self {
            ResponseBody::Lifetime(data) => data,
            ResponseBody::Owned(data) => data.as_slice(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Response<'a> {
    pub version: f32,
    pub status: u16,
    pub content_type: &'static str,
    pub headers: Vec<String>,
    pub body: ResponseBody<'a>,
}

impl<'a> Response<'a> {
    pub fn new() -> Response<'a> {
        return Response {
            version: 1.1,
            status: 200,
            content_type: content_types::PLAIN,
            headers: vec![],
            body: ResponseBody::Lifetime(EMPTY_BODY),
        };
    }

    pub fn send(&self, stream: &mut TcpStream) {
        safe_send(
            stream,
            format!(
                "HTTP/{} {} {}\r\n",
                self.version,
                self.status,
                status_to_message(self.status)
            )
            .as_bytes(),
        );

        safe_send(
            stream,
            format!("Content-Type: {}\r\n", self.content_type).as_bytes(),
        );
        safe_send(stream, "Server: site-3ds\r\n".as_bytes());
        safe_send(stream, "Connection: close\r\n".as_bytes());
        for header in &self.headers {
            safe_send(stream, format!("{}\r\n", header).as_bytes());
        }
        safe_send(
            stream,
            format!("Connection-Length: {}\r\n", self.body.len()).as_bytes(),
        );

        safe_send(stream, "\r\n".as_bytes());

        safe_send(stream, self.body.as_slice());
    }
}

fn status_to_message(status: u16) -> String {
    match status {
        200 => "OK".to_owned(),
        201 => "Created".to_owned(),
        202 => "Accepted".to_owned(),
        204 => "No Content".to_owned(),
        400 => "Bad Request".to_owned(),
        401 => "Unauthorized".to_owned(),
        403 => "Forbidden".to_owned(),
        404 => "Not Found".to_owned(),
        405 => "Method Not Allowed".to_owned(),
        500 => "Internal Server Error".to_owned(),
        _ => "Internal Server Error".to_owned(),
    }
}

pub mod content_types {
    #[allow(dead_code)]
    pub const PLAIN: &'static str = "text/plain";
    #[allow(dead_code)]
    pub const HTML: &'static str = "text/html";
    #[allow(dead_code)]
    pub const JPEG: &'static str = "image/jpeg";
    #[allow(dead_code)]
    pub const PNG: &'static str = "image/png";
    #[allow(dead_code)]
    pub const ICON: &'static str = "image/vnd.microsoft.icon";
    #[allow(dead_code)]
    pub const JSON: &'static str = "application/json";
}

pub const EMPTY_BODY: &'static [u8] = &[];

#[derive(Debug)]
pub struct Request {
    pub method: String,
    pub path: String,
    #[allow(dead_code)]
    pub version: f32,
    #[allow(dead_code)]
    pub headers: Vec<String>,
    #[allow(dead_code)]
    pub body: String,
}

impl Request {
    pub fn from(mut stream: &TcpStream) -> Option<Request> {
        let mut buffer = [0; 8192];
        let mut read_size = 0;
        for _ in 0..10 {
            match stream.read(&mut buffer) {
                Ok(read) => {
                    if read == 0 {
                        break;
                    }
                    read_size += read;
                }
                Err(e) => {
                    if e.kind() == io::ErrorKind::WouldBlock {
                        std::thread::sleep(std::time::Duration::from_millis(10));
                    }
                }
            }
        }

        if read_size == 0 {
            return None;
        }

        let mut buffer_v = Vec::from_iter(buffer.iter().copied());

        buffer_v.pop();
        let raw_req = match std::str::from_utf8(&buffer_v) {
            Ok(raw_req) => raw_req.to_string(),
            Err(e) => {
                println!("Error parsing request: {e}");
                return None;
            }
        };
        let req_handler: Vec<&str> = raw_req.split("\r\n").collect();
        let mut req: Vec<String> = vec![];

        for i in &req_handler {
            req.push(i.to_string())
        }

        let line_one: Vec<&str> = req[0].split(' ').collect();

        let method = line_one[0].to_string();
        let path = line_one[1].to_string();

        let mut trash: Vec<&str> = line_one[2].split('/').collect(); //useless
        let version = trash[1].parse::<f32>().unwrap();

        trash = req[req.len() - 1].split('\0').collect();
        let body = trash[0].to_string();

        req.remove(0);
        req.remove(req.len() - 1);
        req.remove(req.len() - 1);

        let headers: Vec<String> = req.to_vec(); // minus 2 'cause the last is the body and penultimate is "\r\n\r\n"

        Some(Request {
            method,
            path,
            version,
            body,
            headers,
        })
    }
}
