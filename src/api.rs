use actix_web::{get, HttpResponse, Responder, web};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct UrlQuery {
    url: String,
}

#[get("/api/image")]
pub async fn api(info: web::Query<UrlQuery>) -> impl Responder {
    // Ensure this can't be abused.
    let img_path = info.url.as_str().split('/').last().unwrap_or_default();
    let bytes = reqwest::Client::new()
        .get(format!("https://images.genius.com/{}", img_path))
        .header("Authorization", format!("Bearer {}", std::env::var("GENIUS_AUTH_TOKEN").unwrap()))
        .send()
        .await.unwrap().bytes()
        .await.unwrap();
    HttpResponse::Ok().body(bytes)
}
