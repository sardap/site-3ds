use chrono::prelude::*;
use core::net::SocketAddr;
use std::collections::VecDeque;
use std::net::{Shutdown, TcpListener, TcpStream};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::time::Duration;

use crate::api;
use crate::database::Database;
use crate::http_utils::{content_types, Request, Response, ResponseBody, SliceBody};

include!(concat!(env!("OUT_DIR"), "/dist.rs"));

pub struct WorkJob {
    request: Request,
    socket_address: SocketAddr,
    tcp_stream: TcpStream,
}


type JobQueue = Arc<Mutex<VecDeque<WorkJob>>>;

pub struct Worker {
    worker_id: usize,
    db: Arc<Mutex<Database>>,
    queue: JobQueue,
    keep_running: Arc<AtomicBool>,
}

impl Worker {
    pub fn new(worker_id: usize, db: Arc<Mutex<Database>>, queue: JobQueue, keep_running: Arc<AtomicBool>) -> Self {
        Self {worker_id,  db, queue, keep_running }
    }

    fn get_job(&self) -> Option<WorkJob> {
        let mut queue = self.queue.lock().unwrap();
        queue.pop_front()
    }

    fn should_run(&self) -> bool {
        self.keep_running.load(Ordering::Relaxed)
    }

    pub fn work(&mut self) {
        println!("Worker {} started on {}", self.worker_id, std::thread::current().id().as_u64());
        while self.should_run() {
            let mut job = match self.get_job() {
                Some(job) => job,
                None => {
                    std::thread::sleep(Duration::from_millis(50));
                    continue;
                }
            };

            let response = route(&job.request, self.db.clone(), &job.socket_address);
            println!(
                "{} {} {} {} {}\n",
                Utc::now().format("%Y-%m-%d %H:%M:%S"),
                self.worker_id,
                job.request.method,
                job.request.path,
                response.status
            );

            response.send(&mut job.tcp_stream);
            // Shutdown the stream (depending on the web browser used to view the page, this might cause some issues).
            match job.tcp_stream.shutdown(Shutdown::Both) {
                Ok(_) => {}
                Err(e) => {
                    println!("Error shutting down stream: {e}");
                }
            }
        }

        println!("Worker {} running on {} stopped", self.worker_id, std::thread::current().id().as_u64());
    }
}

pub struct Handler {
    server: TcpListener,
    queue: JobQueue,
    index_queue: JobQueue,
    worker_threads: Vec<JoinHandle<()>>,
    keep_running: Arc<AtomicBool>,
}

const QUEUE_MAX_SIZE: usize = 100;

impl Handler {
    pub fn new(db: Arc<Mutex<Database>>, worker_count: usize) -> Self {
        let server = TcpListener::bind("0.0.0.0:8081").unwrap();
        server.set_nonblocking(true).unwrap();

        let keep_running = Arc::new(AtomicBool::new(true));
        let index_queue = JobQueue::default();
        let queue = JobQueue::default();
        
        let mut worker = Worker::new(1, db.clone(), index_queue.clone(), keep_running.clone());
        let thread = std::thread::Builder::new().spawn(move || {
            worker.work();
        }).unwrap();
        let mut worker_threads = vec![thread];
        
        for i in 0..worker_count {
            let mut worker = Worker::new( i + 2, db.clone(), queue.clone(), keep_running.clone());
            let thread = std::thread::Builder::new().spawn(move || {
                worker.work();
            }).unwrap();
            worker_threads.push(thread);
        }

        Self {
            server,
            queue,
            index_queue,
            worker_threads,
            keep_running,
        }
    }

    pub fn stop_workers(&mut self) {
        self.keep_running.store(false, Ordering::Relaxed);
        println!("Sent kill signal to workers");
        for thread in self.worker_threads.drain(..) {
            thread.join().unwrap();
            println!("Worker stopped");
        }
    }

    pub fn step(&mut self) {
        match self.server.accept() {
            Ok((stream, socket_addr)) => {
                // Queue full error out
                let request = match Request::from(&stream) {
                    Some(request) => request,
                    None => {
                        let response = &INTERNAL_SERVER_ERROR;
                        server_error(stream, &response);
                        return;
                    }
                };
                

                let queue = if request.path == "/" {
                    if self.index_queue.lock().unwrap().len() < QUEUE_MAX_SIZE {
                        &self.index_queue                        
                    } else {
                        &self.queue
                    }
                } else {
                    &self.queue
                };
                let job = WorkJob {
                    request,
                    socket_address: socket_addr,
                    tcp_stream: stream,
                };
                {
                    let mut queue = queue.lock().unwrap();
                    if queue.len() >= QUEUE_MAX_SIZE {
                        let response = &SERVICE_UNAVAILABLE;
                        server_error(job.tcp_stream, &response);
                        return;
                    }
                    queue.push_back(job);
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
}

fn server_error(mut stream: TcpStream, response: &Response) {
    response.send(&mut stream);

    // Shutdown the stream (depending on the web browser used to view the page, this might cause some issues).
    match stream.shutdown(Shutdown::Both) {
        Ok(_) => {}
        Err(e) => {
            println!("Error shutting down stream: {e}");
        }
    }
}

fn route<'a>(
    request: &Request,
    db: Arc<Mutex<Database>>,
    socket_address: &SocketAddr,
) -> Response<'a> {
    if request.method == "GET" && request.path == "/" {
        return SERVE_REQUESTS[0].create_response(request);
    }

    if request.path.starts_with("/api/") {
        if let Some(value) = api::route(request, db, socket_address) {
            return value;
        }
    }

    for serve_request in SERVE_REQUESTS.iter() {
        if serve_request.path == request.path {
            if request.method == serve_request.method {
                return serve_request.create_response(request);
            }
            if request.method == "HEAD" {
                return serve_request.create_head_response(request);
            }
        }
    }

    let mut response = Response::new();
    response.status = 404;
    response.content_type = content_types::HTML;
    response.body =
        ResponseBody::Lifetime("<html><body><h1>404 Not Found</h1></body></html>".as_bytes());
    response
}

impl ServeRequest {
    pub fn create_head_response<'a>(&self, request: &Request) -> Response<'a> {
        let mut response = self.create_response(request);
        response.content_length_override = Some(response.body.len());
        response.body = ResponseBody::Empty;
        return response;
    }

    pub fn create_response<'a>(&self, request: &Request) -> Response<'a> {
        let mut response = Response::new();
        response.content_type = self.content_type;
        response.headers.push("Accept-Ranges: bytes".to_string());

        let accept_encoding = request
            .get_header("Accept-Encoding")
            .unwrap_or("".to_string());

        let mut body = (self.body, "");
        if self.body_gzip.is_some()
            && accept_encoding.contains("gzip")
            && self.body_gzip.unwrap().len() < body.0.len()
        {
            body = (self.body_gzip.unwrap(), "gzip");
        }
        if self.body_deflate.is_some()
            && accept_encoding.contains("deflate")
            && self.body_deflate.unwrap().len() < body.0.len()
        {
            body = (self.body_deflate.unwrap(), "deflate");
        }
        if self.body_br.is_some()
            && accept_encoding.contains("br")
            && self.body_br.unwrap().len() < body.0.len()
        {
            body = (self.body_br.unwrap(), "br");
        }
        if self.body_zstd.is_some()
            && accept_encoding.contains("zstd")
            && self.body_zstd.unwrap().len() < body.0.len()
        {
            body = (self.body_zstd.unwrap(), "zstd");
        }
        let (body, encoding) = body;

        response.body = if let Some(range) = request.get_header("Range") {
            let range = range.replace("bytes=", "");
            let (start, end) = if range.ends_with('-') {
                let start = range.trim_end_matches('-').parse::<usize>().unwrap_or(0);
                let end = body.len();
                (start, end)
            } else if range.starts_with('-') {
                let start = 0;
                let end = body.len() - range.trim_start_matches('-').parse::<usize>().unwrap_or(body.len());
                (start, end)
            } else {
                let mut parts = range.split('-');
                let start = parts.next().unwrap().parse::<usize>().unwrap_or(0);
                let end = parts.next().unwrap().parse::<usize>().unwrap_or(body.len());
                (start, end)
            };

            response.status = 206;
            response.headers.push(format!("Content-Range: bytes {}-{}/{}", start, end - 1, body.len()));
            ResponseBody::Slice(SliceBody{
                data: body,
                start,
                end,
            })
        } else {
            ResponseBody::Lifetime(body)
        };
        if !encoding.is_empty() {
            response
                .headers
                .push(format!("Content-Encoding: {}", encoding));
        }

        return response;
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
    content_length_override: None,
};

const SERVICE_UNAVAILABLE: Response = Response {
    version: 1.1,
    status: 503,
    content_type: content_types::HTML,
    headers: vec![],
    body: ResponseBody::Lifetime(
        "<html><body><h1>503 Service Unavailable</h1></body></html>".as_bytes(),
    ),
    content_length_override: None,
};