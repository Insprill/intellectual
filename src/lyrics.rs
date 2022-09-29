use actix_web::{get, Responder, web};
use askama::Template;
use scraper::{Html, Selector};
use serde::Deserialize;

use crate::genius;
use crate::templates::template;

struct Verse {
    title: String,
    lyrics: Vec<String>,
}

#[derive(Template)]
#[template(path = "lyrics.html")]
struct LyricsTemplate {
    verses: Vec<Verse>,
    query: LyricsQuery,
    song: GeniusSong,
}

#[derive(Debug, Deserialize)]
pub struct LyricsQuery {
    path: String,
    api_path: String,
}

#[get("/lyrics")]
pub async fn lyrics(info: web::Query<LyricsQuery>) -> impl Responder {
    let api_response = genius::text(genius::SubDomain::Api, info.api_path.trim_start_matches('/')).await;
    let api: GeniusRequest = serde_json::from_str(&api_response).unwrap();
    let lyric_response = genius::text(genius::SubDomain::Root, info.path.trim_start_matches('/')).await;
    let verses = scrape_lyrics(&lyric_response);
    template(LyricsTemplate { verses, query: info.into_inner(), song: api.response.song })
}

fn scrape_lyrics(doc: &str) -> Vec<Verse> {
    let document = Html::parse_document(doc);
    let parser = &Selector::parse("div[data-lyrics-container=true]").unwrap();

    let mut verses: Vec<Verse> = Vec::new();

    for x in document.select(parser).flat_map(|x| x.text()) {
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

    if verses.is_empty() {
        verses.push(Verse { title: "This song has no lyrics".to_string(), lyrics: Vec::new() })
    }

    verses
}

#[derive(Deserialize)]
struct GeniusRequest {
    response: GeniusResponse,
}

#[derive(Deserialize)]
struct GeniusResponse {
    song: GeniusSong,
}

#[derive(Deserialize)]
struct GeniusSong {
    title: String,
    artist_names: String,
    description: GeniusDescription,
    header_image_url: String,
}

#[derive(Deserialize)]
struct GeniusDescription {
    plain: String,
}
