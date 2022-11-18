use actix_web::{get, web, HttpResponse, Responder};
use reqwest::StatusCode;
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
    let response = genius::request(genius::SubDomain::Images, img_path).await;
    if response.status() != StatusCode::OK {
        return HttpResponse::build(response.status()).await.unwrap();
    }
    HttpResponse::Ok().body(response.bytes().await.unwrap())
}
