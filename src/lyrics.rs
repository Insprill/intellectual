use actix_web::{get, HttpResponse, Responder, web};
use scraper::{Html, Selector};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct SearchQuery {
    path: String,
}

#[get("/lyrics")]
pub async fn lyrics(info: web::Query<SearchQuery>) -> impl Responder {
    let body = reqwest::Client::new()
        .get(format!("https://genius.com/{}", info.path))
        .header("Authorization", format!("Bearer {}", std::env::var("AUTH_TOKEN").unwrap()))
        .send()
        .await.unwrap().text_with_charset("utf-8")
        .await.unwrap();
    let x = scrape_lyrics(body);
    HttpResponse::Ok().body(x)
}

fn scrape_lyrics(doc: String) -> String {
    let document = Html::parse_document(&doc);
    let mut data: String = "".to_string();

    let parser = &Selector::parse("div[data-lyrics-container=true]").unwrap();

    for element in document.select(parser) {
        data.push_str(&element.html());
    }

    data
}
