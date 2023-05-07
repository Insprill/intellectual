use actix_web::{get, web, Responder, Result};
use askama::Template;
use futures::future;
use once_cell::sync::Lazy;
use scraper::{Html, Selector};
use serde::Deserialize;

use crate::genius::GeniusSong;
use crate::genius::{self, GeniusApi};
use crate::templates::template;
use crate::utils;

static SONG_ID_SELECTOR: Lazy<Selector> =
    Lazy::new(|| Selector::parse("meta[property='twitter:app:url:iphone']").unwrap());
static LYRIC_SELECTOR: Lazy<Selector> =
    Lazy::new(|| Selector::parse("div[data-lyrics-container=true]").unwrap());

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
    id: Option<u32>,
    path: String,
}

#[get("/lyrics")]
pub async fn lyrics(info: web::Query<LyricsQuery>) -> Result<impl Responder> {
    let document: Html;
    let song: GeniusSong;

    if let Some(id) = info.id {
        let responses = future::join(
            GeniusApi::global().get_text(genius::SubDomain::Root, &info.path, None),
            GeniusApi::global().get_song(id),
        )
        .await;
        document = Html::parse_document(&responses.0?);
        song = responses.1?;
    } else {
        let lyric_page = GeniusApi::global()
            .get_text(genius::SubDomain::Root, &info.path, None)
            .await?;
        document = Html::parse_document(&lyric_page);
        let id = get_song_id(&document)?;
        song = GeniusApi::global().get_song(id).await?;
    }

    let verses = scrape_lyrics(&document);

    Ok(template(LyricsTemplate {
        verses,
        query: info.0,
        song,
    }))
}

fn get_song_id(document: &Html) -> crate::Result<u32> {
    let meta = document
        .select(&SONG_ID_SELECTOR)
        .next()
        .ok_or("Failed to find meta tag with song ID")?;
    let id = meta
        .value()
        .attr("content")
        .and_then(|content| content.strip_prefix("genius://songs/"))
        .ok_or("Failed to find content attribute")?
        .parse::<u32>()?;
    Ok(id)
}

fn scrape_lyrics(document: &Html) -> Vec<Verse> {
    let text_iter = document.select(&LYRIC_SELECTOR).flat_map(|x| x.text());

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
