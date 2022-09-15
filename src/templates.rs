use actix_web::{HttpResponse, Responder};
use askama::Template;

pub fn template(t: impl Template) -> impl Responder {
    HttpResponse::Ok()
        .append_header(("Content-Type", "text/html; charset=utf-8"))
        .body(t.render().unwrap_or_default())
}
