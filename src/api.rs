use std::io::{BufWriter, Cursor};

use ::image::{DynamicImage, EncodableLayout, ImageFormat, imageops::FilterType, load_from_memory};
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
    // Songs without art point to a placeholder on assets.genius.com, which images.genius.com refuses.
    let (subdomain, img_path) = match info.url.strip_prefix("https://assets.genius.com/") {
        Some(path) => (SubDomain::Assets, path),
        None => match info.url.split('/').next_back() {
            Some(path) => (SubDomain::Images, path),
            None => return Ok(HttpResponse::BadRequest().finish()),
        },
    };

    let (status, body, headers) = genius::get_raw(subdomain, img_path, None).await?;

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
        // Images typically aren't smaller than 2kb
        let mut buf = BufWriter::new(Cursor::new(Vec::with_capacity(2048)));
        let resized = resize(abstract_image, info.size);

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

const MAX_SIZE: u32 = 1000;

fn resize(img: DynamicImage, size: Option<u32>) -> DynamicImage {
    match size {
        Some(size) => {
            let size = size.clamp(1, MAX_SIZE);
            img.resize_exact(size, size, FilterType::Nearest)
        }
        None if img.width() > MAX_SIZE || img.height() > MAX_SIZE => {
            img.resize(MAX_SIZE, MAX_SIZE, FilterType::Nearest)
        }
        None => img,
    }
}

fn send_image(bytes: Vec<u8>, content_type: &'static str) -> Result<HttpResponse> {
    Ok(HttpResponse::Ok()
        .append_header(("Cache-Control", "public, max-age=31536000, immutable"))
        .append_header(("Content-Type", content_type))
        .body(bytes))
}

#[cfg(test)]
mod tests {
    use ::image::GenericImageView;

    use super::*;

    #[test]
    fn resize_keeps_dimensions_without_size() {
        let resized = resize(DynamicImage::new_rgb8(900, 600), None);
        assert_eq!(resized.dimensions(), (900, 600));
    }

    #[test]
    fn resize_fits_large_images_within_cap() {
        let resized = resize(DynamicImage::new_rgb8(600, 6000), None);
        assert_eq!(resized.dimensions(), (100, 1000));
    }

    #[test]
    fn resize_caps_requested_size() {
        let resized = resize(DynamicImage::new_rgb8(1, 4000), Some(4000));
        assert_eq!(resized.dimensions(), (MAX_SIZE, MAX_SIZE));
    }

    #[test]
    fn resize_makes_square_thumbnails() {
        assert_eq!(
            resize(DynamicImage::new_rgb8(300, 300), Some(150)).dimensions(),
            (150, 150)
        );
        assert_eq!(
            resize(DynamicImage::new_rgb8(300, 200), Some(0)).dimensions(),
            (1, 1)
        );
    }
}
