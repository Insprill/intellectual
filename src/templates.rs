use actix_web::HttpResponse;
use askama::Template;

pub fn template(t: impl Template) -> HttpResponse {
    HttpResponse::Ok()
        .append_header(("Content-Type", "text/html; charset=utf-8"))
        // Caching Setup
        // Since Cloudflare ignores Vary headers, we can't publically cache all pages since only
        // the last-cached theme would be shown to users. Instead, we privately cache all pages in the
        // browser, which does handle the Vary header correctly. If we didn't have the Vary header,
        // when a user changes themes, it wouldn't be applied to previously visited pages (e.g. the
        // home page) until the browser requests the page from the server again.
        .append_header(("Vary", "settings"))
        .append_header(("Cache-Control", "private, max-age=604800"))
        .body(t.render().unwrap_or_default())
}
