use actix_web::{HttpResponse, Responder, get, web};
use include_dir::{Dir, include_dir};

const STATIC_RESOURCES: Dir = include_dir!("$CARGO_MANIFEST_DIR/static");

#[get("{filename:.*}")]
pub async fn resource(path: web::Path<String>) -> impl Responder {
    asset(path.as_str())
}

fn asset(path: &str) -> impl Responder + use<> {
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
    match path.split('.').next_back().unwrap_or_default() {
        "css" => "text/css",
        "svg" => "image/svg+xml",
        "woff2" => "font/woff2",
        "json" => "application/json",
        _ => "text/plain",
    }
}
