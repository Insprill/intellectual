use actix_web::{get, http::StatusCode, web, HttpResponse, Responder, Result};
use serde::Deserialize;

use crate::genius::{GeniusApi, SubDomain};

#[derive(Debug, Deserialize)]
pub struct UrlQuery {
    url: String,
}

#[get("/api/image")]
pub async fn image(info: web::Query<UrlQuery>) -> Result<impl Responder> {
    match info.url.split('/').last() {
        Some(img_path) => {
            let (status, body) = GeniusApi::global()
                .get_raw(SubDomain::Images, img_path, None)
                .await?;

            if status != StatusCode::OK {
                return Ok(HttpResponse::build(status).finish());
            }

            Ok(HttpResponse::Ok()
                .append_header(("Cache-Control", "max-age=604800"))
                .body(body))
        }
        None => Ok(HttpResponse::BadRequest().finish()),
    }
}
