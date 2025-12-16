use actix_web::{HttpResponse, HttpResponseBuilder};
use askama::Template;

pub fn template(t: impl Template) -> HttpResponse {
    template_with_res(HttpResponse::Ok(), t)
}

pub fn template_with_res(mut res: HttpResponseBuilder, t: impl Template) -> HttpResponse {
    res.append_header(("Content-Type", "text/html; charset=utf-8"))
        // Caching Setup
        // Since Cloudflare ignores Vary headers, we can't publicly cache all pages since only
        // the last-cached theme would be shown to users. Instead, we privately cache all pages in the
        // browser, which does handle the Vary header correctly. If we didn't have the Vary header,
        // when a user changes themes, it won't be applied to previously visited pages (e.g. the
        // homepage) until the browser requests the page from the server again.
        .append_header(("Vary", "Cookie"))
        .append_header(("Cache-Control", "private, max-age=604800"));

    res.body(t.render().unwrap_or_default())
}
