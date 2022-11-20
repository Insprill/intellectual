use actix_web::{get, http::StatusCode, web, HttpResponse, Responder};
use serde::Deserialize;

use crate::genius;

#[derive(Debug, Deserialize)]
pub struct UrlQuery {
    url: String,
}

#[get("/api/image")]
pub async fn api(info: web::Query<UrlQuery>) -> impl Responder {
    // Ensure this can't be abused.
    let img_path = info.url.as_str().split('/').last().unwrap_or_default();
    let (status, body) = genius::request(genius::SubDomain::Images, img_path, None).await;
    if status != StatusCode::OK {
        return HttpResponse::build(status).await.unwrap();
    }
    HttpResponse::Ok().body(body)
}
