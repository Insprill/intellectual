use actix_web::{get, HttpResponse, Responder, web};
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
        res.push_str(&format!("{}\n\n", hit.result.full_title));
    }

    HttpResponse::Ok().append_header(("Content-Type", "text/plain; charset=utf-8")).body(res)
}

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
    artist_names: String,
    full_title: String,
    id: u64,
    language: String,
    //TODO: rest of these
}
