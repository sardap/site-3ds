use std::{collections::HashMap, net::IpAddr, str::FromStr, sync::{Arc, Mutex}};

use core::net::SocketAddr;

use crate::{
    database::Database,
    http_utils::{Request, Response, ResponseBody, content_types},
};
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
struct ApiResponse<T: Serialize> {
    data: T,
}

impl<T: Serialize> ApiResponse<T> {
    fn new<'a>(data: T) -> ResponseBody<'a> {
        ResponseBody::Owned(serde_json::to_vec(&ApiResponse { data }).unwrap())
    }
}

#[derive(Serialize)]
struct NotFoundResponse {
    message: String,
}

#[derive(Serialize)]
pub struct ClicksResponse {
    pub clicks: u64,
}

#[derive(Serialize)]
pub struct VisitsResponse {
    pub visits: u64,
}

#[derive(Serialize)]
pub struct ReviewRatingsResponse {
    pub review_ratings: HashMap<u8, i64>,
}

#[derive(Serialize)]
pub struct ReviewRatingResponse {
    pub id: u8,
    pub rating: i64,
}

#[derive(Deserialize)]
pub struct PostReviewRatingRequest {
    pub id: u8,
    pub positive: bool,
}

pub fn route<'a>(
    request: &Request,
    db: Arc<Mutex<Database>>,
    socket_addr: &SocketAddr,
) -> Option<Response<'a>> {
    if request.method == "GET" && request.path == "/api/review_ratings" {
        let db = db.lock().unwrap();
        let mut response = Response::new();
        response.content_type = content_types::JSON;
        response.body = ApiResponse::new(ReviewRatingsResponse {
            review_ratings: db.get_review_ratings(),
        });
        return Some(response);
    }

    if request.method == "POST" && request.path == "/api/review_ratings" {
        let mut db = db.lock().unwrap();
        let response = match serde_json::from_str::<PostReviewRatingRequest>(&request.body) {
            Ok(request_body) => {
                let mut response = Response::new();
                response.content_type = content_types::JSON;
                db.add_review_rating(request_body.id, if request_body.positive { 1 } else { -1 });
                response.body = ApiResponse::new(ReviewRatingResponse {
                    id: request_body.id,
                    rating: db.get_review_rating(request_body.id),
                });
                response
            }
            Err(e) => {
                let mut response = Response::new();
                response.status = 400;
                response.content_type = content_types::JSON;
                response.body = ApiResponse::new(NotFoundResponse {
                    message: format!("Invalid request body: {e}"),
                });
                response
            }
        };
        return Some(response);
    }

    if request.method == "GET" && request.path == "/api/visits" {
        let mut db = db.lock().unwrap();
        let ip = match request.get_header("X-Forwarded-For") {
            Some(value) => {
                println!("X-Forwarded-For: {value}");
                match IpAddr::from_str(&value) {
                    Ok(ip_address) => ip_address,
                    Err(e) => {
                        println!("Error parsing IP address: {e}");
                        socket_addr.ip()
                    }
                }
            }
            None => socket_addr.ip(),
        };

        db.add_visit(&ip);

        let mut response = Response::new();
        response.content_type = content_types::JSON;
        response.body = ApiResponse::new(VisitsResponse {
            visits: db.get_visits(),
        });
        return Some(response);
    }

    None
}
