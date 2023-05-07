use actix_web::{get, web, Responder, Result};
use askama::Template;
use futures::future;
use scraper::{Html, Selector};
use serde::Deserialize;

use crate::genius::GeniusSong;
use crate::genius::{self, GeniusApi};
use crate::templates::template;
use crate::utils;

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
    id: u32,
    path: String,
}

#[get("/lyrics")]
pub async fn lyrics(info: web::Query<LyricsQuery>) -> Result<impl Responder> {
    let responses = future::join(
        GeniusApi::global().get_song(info.id),
        GeniusApi::global().get_text(genius::SubDomain::Root, &info.path, None),
    )
    .await;

    let song = responses.0?;
    let verses = scrape_lyrics(&responses.1?);

    Ok(template(LyricsTemplate {
        verses,
        query: info.0,
        song,
    }))
}

fn scrape_lyrics(doc: &str) -> Vec<Verse> {
    let document = Html::parse_document(doc);
    let parser = &Selector::parse("div[data-lyrics-container=true]").unwrap();

    let text_iter = document.select(parser).flat_map(|x| x.text());

    let mut verses = Vec::with_capacity(text_iter.size_hint().0);

    for text in text_iter {
        if text.starts_with('[') && text.ends_with(']') {
            verses.push(Verse {
                title: text.to_string(),
                lyrics: Vec::new(),
            });
            continue;
        }
        let trimmed = text.trim();
        if trimmed.is_empty() {
            continue;
        }
        if verses.is_empty() {
            verses.push(Verse {
                title: String::new(),
                lyrics: Vec::new(),
            })
        }
        let idx = verses.len() - 1;
        if let Some(verse) = verses.get_mut(idx) {
            verse.lyrics.push(trimmed.to_owned())
        }
    }

    if verses.is_empty() {
        verses.push(Verse {
            title: "This song has no lyrics".to_owned(),
            lyrics: Vec::new(),
        })
    }

    verses
}
