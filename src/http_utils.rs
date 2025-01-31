use std::{
    io::{self, Read, Write},
    net::TcpStream,
};

fn safe_send(stream: &mut TcpStream, data: &[u8]) -> Result<(), ()> {
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
                    return Err(());
                }
            }
        };
        offset += write;
        if offset >= data.len() {
            return Ok(());
        }
    }
}

#[derive(Clone, Debug)]
pub struct SliceBody<'a> {
    pub data: &'a [u8],
    pub start: usize,
    pub end: usize,
}

#[derive(Clone, Debug)]
pub enum ResponseBody<'a> {
    Lifetime(&'a [u8]),
    Slice(SliceBody<'a>),
    Owned(Vec<u8>),
    Empty,
}

impl ResponseBody<'_> {
    pub fn len(&self) -> usize {
        match self {
            ResponseBody::Lifetime(data) => data.len(),
            ResponseBody::Owned(data) => data.len(),
            ResponseBody::Slice(slice) => slice.end - slice.start,
            ResponseBody::Empty => 0,
        }
    }

    pub fn chunks<'a>(&self, chunk_size: usize) -> std::slice::Chunks<'_, u8> {
        match self {
            ResponseBody::Lifetime(data) => data.chunks(chunk_size),
            ResponseBody::Owned(data) => data.chunks(chunk_size),
            ResponseBody::Slice(slice) => slice.data[slice.start..slice.end].chunks(chunk_size),
            ResponseBody::Empty => EMPTY_BODY.chunks(chunk_size),
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
    pub content_length_override: Option<usize>,
}

impl<'a> Response<'a> {
    pub fn new() -> Response<'a> {
        return Response {
            version: 1.1,
            status: 200,
            content_type: content_types::PLAIN,
            headers: vec![],
            body: ResponseBody::Lifetime(EMPTY_BODY),
            content_length_override: None,
        };
    }

    pub fn send(&self, stream: &mut TcpStream) {
        {
            let mut send_body = String::with_capacity(256);
            send_body.push_str(&format!(
                "HTTP/{} {} {}\r\n",
                self.version,
                self.status,
                status_to_message(self.status)
            ));
            send_body.push_str("Server: site-3ds\r\n");
            send_body.push_str(&format!("Content-Type: {}\r\n", self.content_type));
            send_body.push_str(&format!(
                "Content-Length: {}\r\n",
                if let Some(len) = self.content_length_override {
                    len
                } else {
                    self.body.len()
                }
            ));
            // send_body.push_str("Connection: close\r\n");
            for header in &self.headers {
                send_body.push_str(&format!("{}\r\n", header));
            }
            send_body.push_str("\r\n");
            if safe_send(stream, send_body.as_bytes()).is_err() {
                return;
            }
        }

        for chunk in self.body.chunks(2000) {
            if safe_send(stream, chunk).is_err() {
                return;
            }
        }
    }
}

fn status_to_message(status: u16) -> String {
    match status {
        200 => "OK".to_owned(),
        201 => "Created".to_owned(),
        202 => "Accepted".to_owned(),
        204 => "No Content".to_owned(),
        205 => "Reset Content".to_owned(),
        206 => "Partial Content".to_owned(),
        400 => "Bad Request".to_owned(),
        401 => "Unauthorized".to_owned(),
        403 => "Forbidden".to_owned(),
        404 => "Not Found".to_owned(),
        405 => "Method Not Allowed".to_owned(),
        500 => "Internal Server Error".to_owned(),
        503 => "Service Unavailable".to_owned(),
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

    pub fn get_header(&self, header: &str) -> Option<String> {
        for h in &self.headers {
            let mut splitter = h.splitn(2, ": ");
            if let (Some(first), Some(second)) = (splitter.next(), splitter.next()) {
                if first == header {
                    return Some(second.to_string());
                }
            }
        }
        None
    }
}
