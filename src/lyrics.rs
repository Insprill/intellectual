use actix_web::{get, Responder, web};
use askama::Template;
use scraper::{Html, Selector};
use serde::Deserialize;

use crate::templates::template;

struct Verse {
    title: String,
    lyrics: Vec<String>,
}

#[derive(Template)]
#[template(path = "lyrics.html")]
struct LyricsTemplate {
    verses: Vec<Verse>,
}

#[derive(Debug, Deserialize)]
pub struct LyricsQuery {
    path: String,
}

#[get("/lyrics")]
pub async fn lyrics(info: web::Query<LyricsQuery>) -> impl Responder {
    let body = reqwest::Client::new()
        .get(format!("https://genius.com/{}", info.path.trim_start_matches("/")))
        .header("Authorization", format!("Bearer {}", std::env::var("AUTH_TOKEN").unwrap()))
        .send()
        .await.unwrap().text_with_charset("utf-8")
        .await.unwrap();
    let x = scrape_lyrics(body);
    template(LyricsTemplate { verses: x })
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
