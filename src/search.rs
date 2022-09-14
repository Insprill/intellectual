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
        .await
        .unwrap().text()
        .await;
    HttpResponse::Ok().body(body.unwrap())
}