use actix_web::{get, web, Responder, Result};
use askama::Template;
use futures::future;
use scraper::{Html, Selector};
use serde::Deserialize;

use crate::genius;
use crate::genius::{GeniusSong, GeniusSongRequest};
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
    path: String,
    api_path: String,
}

#[get("/lyrics")]
pub async fn lyrics(info: web::Query<LyricsQuery>) -> Result<impl Responder> {
    let trimmed_api_path = info.api_path.trim_start_matches('/');

    let responses = future::join(
        genius::get_text(genius::SubDomain::Api, trimmed_api_path, None),
        genius::get_text(
            genius::SubDomain::Root,
            info.path.trim_start_matches('/'),
            None,
        ),
    )
    .await;

    let api: GeniusSongRequest = serde_json::from_str(&responses.0?)?;
    let verses = scrape_lyrics(&responses.1?);

    Ok(template(LyricsTemplate {
        verses,
        query: info.into_inner(),
        song: api.response.song,
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
            })
        } else {
            if verses.is_empty() {
                verses.push(Verse {
                    title: "".to_string(),
                    lyrics: Vec::new(),
                })
            }
            let mut x1 = verses.remove(verses.len() - 1);
            x1.lyrics.push(text.to_string());
            verses.push(x1);
        }
    }

    if verses.is_empty() {
        verses.push(Verse {
            title: "This song has no lyrics".to_string(),
            lyrics: Vec::new(),
        })
    }

    verses
}
