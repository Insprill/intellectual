use actix_web::{get, web, HttpResponse, Responder};
use include_dir::{include_dir, Dir};

const STATIC_RESOURCES: Dir = include_dir!("$CARGO_MANIFEST_DIR/static");

#[get("{resource}")]
pub async fn resource(path: web::Path<String>) -> impl Responder {
    asset(path.as_str())
}

#[get("style/{resource}")]
pub async fn style(path: web::Path<String>) -> impl Responder {
    asset(&format!("style/{}", path.as_str()))
}

#[get("style/theme/{resource}")]
pub async fn style_theme(path: web::Path<String>) -> impl Responder {
    asset(&format!("style/theme/{}", path.as_str()))
}

#[get("icon/{resource}")]
pub async fn icon(path: web::Path<String>) -> impl Responder {
    asset(&format!("icon/{}", path.as_str()))
}

#[get("font/{resource}")]
pub async fn font(path: web::Path<String>) -> impl Responder {
    asset(&format!("font/{}", path.as_str()))
}

fn asset(path: &str) -> impl Responder {
    let file = match STATIC_RESOURCES.get_file(path) {
        Some(file) => file,
        None => return HttpResponse::NotFound().finish(),
    };
    HttpResponse::Ok()
        .append_header(("Content-Type", content_type(path)))
        .append_header(("Cache-Control", "public, max-age=31536000, immutable"))
        .body(file.contents())
}

fn content_type(path: &str) -> &str {
    return match path.split('.').last().unwrap_or_default() {
        "css" => "text/css",
        "svg" => "image/svg+xml",
        "woff2" => "font/woff2",
        "json" => "application/json",
        _ => "text/plain",
    };
}
