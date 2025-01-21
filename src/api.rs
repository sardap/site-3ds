use std::collections::HashMap;

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
pub struct ReviewRatingsResponse {
    pub review_ratings: HashMap<u8, i128>,
}

#[derive(Serialize)]
pub struct ReviewRatingResponse {
    pub id: u8,
    pub rating: i128,
}

#[derive(Deserialize)]
pub struct PostReviewRatingRequest {
    pub id: u8,
    pub positive: bool,
}

pub fn route<'a>(request: &Request, db: &mut Database) -> Option<Response<'a>> {
    if request.method == "GET" && request.path == "/api/clicks" {
        let mut response = Response::new();
        response.content_type = content_types::JSON;
        response.body = ApiResponse::new(ClicksResponse {
            clicks: db.clicks(),
        });
        return Some(response);
    }

    if request.method == "POST" && request.path == "/api/clicks" {
        let mut response = Response::new();
        response.content_type = content_types::JSON;
        db.set_clicks(db.clicks() + 1);
        response.body = ApiResponse::new(ClicksResponse {
            clicks: db.clicks(),
        });
        return Some(response);
    }

    if request.method == "GET" && request.path == "/api/review_ratings" {
        let mut response = Response::new();
        response.content_type = content_types::JSON;
        response.body = ApiResponse::new(ReviewRatingsResponse {
            review_ratings: db.get_review_ratings(),
        });
        return Some(response);
    }

    if request.method == "POST" && request.path == "/api/review_ratings" {
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

    None
}
