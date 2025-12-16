use std::io::{BufWriter, Cursor};

use ::image::{EncodableLayout, ImageFormat, imageops::FilterType, load_from_memory};
use actix_web::{HttpRequest, HttpResponse, Responder, get, http::StatusCode, http::header, web};
use serde::Deserialize;

use crate::Result;
use crate::genius::{self, SubDomain};

#[derive(Debug, Deserialize)]
pub struct UrlQuery {
    url: String,
    size: Option<u32>,
}

#[get("/api/image")]
pub async fn image(req: HttpRequest, info: web::Query<UrlQuery>) -> Result<impl Responder> {
    let img_path = match info.url.split('/').next_back() {
        Some(path) => path,
        None => return Ok(HttpResponse::BadRequest().finish()),
    };

    let (status, body, headers) = genius::get_raw(SubDomain::Images, img_path, None).await?;

    if status != StatusCode::OK {
        return Ok(HttpResponse::build(status).finish());
    }

    // Directly pass through GIFs
    if let Some(content_type) = headers.get("Content-Type")
        && content_type.to_str()? == "image/gif"
    {
        return send_image(body.as_bytes().to_vec(), "image/gif");
    }

    let supports_webp = req
        .headers()
        .get(header::ACCEPT)
        .map(|value| value.to_str().unwrap_or_default().contains("webp"))
        .unwrap_or(false);

    if let Ok(abstract_image) = load_from_memory(body.as_bytes()) {
        let size = info
            .size
            .unwrap_or(abstract_image.height())
            .clamp(1, abstract_image.height().max(1000));

        // Images typically aren't smaller than 2kb
        let mut buf = BufWriter::new(Cursor::new(Vec::with_capacity(2048)));
        let resized = abstract_image.resize_exact(size, size, FilterType::Nearest);

        let image_format = if supports_webp {
            ImageFormat::WebP
        } else {
            ImageFormat::Jpeg
        };

        if resized.write_to(&mut buf, image_format).is_ok() {
            let bytes = buf.into_inner().unwrap().into_inner(); // Should never error
            return send_image(
                bytes,
                if supports_webp {
                    "image/webp"
                } else {
                    "image/jpeg"
                },
            );
        }
    }

    Ok(HttpResponse::InternalServerError().finish())
}

fn send_image(bytes: Vec<u8>, content_type: &'static str) -> Result<HttpResponse> {
    Ok(HttpResponse::Ok()
        .append_header(("Cache-Control", "public, max-age=31536000, immutable"))
        .append_header(("Content-Type", content_type))
        .body(bytes))
}
