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
        .header("Authorization", format!("Bearer {}", std::env::var("GENIUS_AUTH_TOKEN").unwrap()))
        .send()
        .await.unwrap().text_with_charset("utf-8")
        .await.unwrap();
    let x = scrape_lyrics(body);
    template(LyricsTemplate { verses: x })
}

fn scrape_lyrics(doc: String) -> Vec<Verse> {
    let document = Html::parse_document(&doc);
    let parser = &Selector::parse("div[data-lyrics-container=true]").unwrap();

    let mut verses: Vec<Verse> = Vec::new();

    for x in document.select(parser).flat_map(|x| x.text()).into_iter() {
        if x.starts_with('[') && x.ends_with(']') {
            verses.push(Verse { title: x.to_string(), lyrics: Vec::new() })
        } else {
            if verses.is_empty() {
                verses.push(Verse { title: "".to_string(), lyrics: Vec::new() })
            }
            let mut x1 = verses.remove(verses.len() - 1);
            x1.lyrics.push(x.to_string());
            verses.push(x1);
        }
    }

    verses
}
