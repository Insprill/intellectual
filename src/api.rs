use actix_web::{get, http::StatusCode, web, HttpResponse, Responder, Result};
use serde::Deserialize;

use crate::genius;

#[derive(Debug, Deserialize)]
pub struct UrlQuery {
    url: String,
}

#[get("/api/image")]
pub async fn image(info: web::Query<UrlQuery>) -> Result<impl Responder> {
    let img_path = &info.url.split('/').last().unwrap_or_default();
    let (status, body) = genius::get(genius::SubDomain::Images, img_path, None).await?;

    if status != StatusCode::OK {
        return Ok(HttpResponse::build(status).finish());
    }

    Ok(HttpResponse::Ok().body(body))
}
