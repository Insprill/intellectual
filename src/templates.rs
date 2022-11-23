use actix_web::HttpResponse;
use askama::Template;

pub fn template(t: impl Template) -> HttpResponse {
    HttpResponse::Ok()
        .append_header(("Content-Type", "text/html; charset=utf-8"))
        .body(t.render().unwrap_or_default())
}
