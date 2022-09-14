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
        .await.unwrap().text()
        .await.unwrap();
    let deserialized: GeniusSearch = serde_json::from_str(&body).unwrap();
    let x = deserialized.response.hits.into_iter().nth(0).unwrap();
    HttpResponse::Ok().body(x.result.full_title)
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
