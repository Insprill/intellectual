use std::io::{BufWriter, Cursor};

use ::image::{imageops::FilterType, load_from_memory, EncodableLayout, ImageFormat};
use actix_web::{get, http::header, http::StatusCode, web, HttpRequest, HttpResponse, Responder};
use serde::Deserialize;

use crate::genius::{self, SubDomain};
use crate::Result;

#[derive(Debug, Deserialize)]
pub struct UrlQuery {
    url: String,
    size: Option<u16>,
}

#[get("/api/image")]
pub async fn image(req: HttpRequest, info: web::Query<UrlQuery>) -> Result<impl Responder> {
    let img_path = match info.url.split('/').last() {
        Some(path) => path,
        None => return Ok(HttpResponse::BadRequest().finish()),
    };

    let (status, body, _) = genius::get_raw(SubDomain::Images, img_path, None).await?;

    if status != StatusCode::OK {
        return Ok(HttpResponse::build(status).finish());
    }

    let webp = req
        .headers()
        .get(header::ACCEPT)
        .map(|value| value.to_str().unwrap_or_default().contains("webp"))
        .unwrap_or(false);

    let size = info.size.unwrap_or(150).clamp(1, 500).into();

    if let Ok(abstract_image) = load_from_memory(body.as_bytes()) {
        // Images typically aren't smaller than 2kb
        let mut buf = BufWriter::new(Cursor::new(Vec::with_capacity(2048)));
        let resized = abstract_image.resize_exact(size, size, FilterType::Nearest);

        let image_format = if webp {
            ImageFormat::WebP
        } else {
            ImageFormat::Jpeg
        };

        if resized.write_to(&mut buf, image_format).is_ok() {
            let bytes = buf.into_inner().unwrap().into_inner(); // Should never error
            return Ok(HttpResponse::Ok()
                .append_header(("Cache-Control", "public, max-age=31536000, immutable"))
                .append_header((
                    "Content-Type",
                    if webp { "image/webp" } else { "image/jpeg" },
                ))
                .body(bytes));
        }
    }

    Ok(HttpResponse::InternalServerError().finish())
}
