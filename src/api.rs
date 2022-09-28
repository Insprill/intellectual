use actix_web::{get, HttpResponse, Responder, web};
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
    let response = genius::bytes(genius::SubDomain::Images, img_path).await;
    HttpResponse::Ok().body(response)
}
