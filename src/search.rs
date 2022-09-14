use actix_web::{get, HttpResponse, Responder, web};
use askama::Template;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct SearchQuery {
    q: String,
}

#[get("/search")]
pub async fn search(info: web::Query<SearchQuery>) -> impl Responder {
    let body = reqwest::Client::new()
        .get(format!("https://api.genius.com/search?q={}", info.q))
        .header("Authorization", format!("Bearer {}", std::env::var("AUTH_TOKEN").unwrap()))
        .send()
        .await.unwrap().text_with_charset("utf-8")
        .await.unwrap();
    let deserialized: GeniusSearch = serde_json::from_str(&body).unwrap();

    let mut res: String = "".to_string();

    for hit in deserialized.response.hits {
        res.push_str(&format!("<a href=\"lyrics?path={}\">{}</a><br/>", hit.result.path, hit.result.full_title));
    }

    HttpResponse::Ok().append_header(("Content-Type", "text/html; charset=utf-8")).body(res)
}

// region Genius Response
#[derive(Deserialize)]
struct GeniusSearch {
    response: GeniusResponse,
}

#[derive(Deserialize)]
struct GeniusResponse {
    hits: Vec<GeniusHit>,
}

#[derive(Deserialize)]
struct GeniusHit {
    result: GeniusResult,
}

#[allow(dead_code)]
#[derive(Deserialize)]
struct GeniusResult {
    full_title: String,
    path: String,
}
// endregion

#[derive(Template)]
#[template(path = "search.html")]
struct SearchTemplate {}
