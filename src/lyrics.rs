use std::sync::LazyLock;

use actix_web::{get, web, HttpRequest, Responder, Result};
use askama::Template;
use futures::future;

use scraper::{Html, Node, Selector};
use serde::Deserialize;

use crate::genius::GeniusSong;
use crate::genius::{self};
use crate::settings::{settings_from_req, Settings};
use crate::templates::template;
use crate::utils;

static SONG_ID_SELECTOR: LazyLock<Selector> =
    LazyLock::new(|| Selector::parse("meta[property='twitter:app:url:iphone']").unwrap());
static LYRIC_SELECTOR: LazyLock<Selector> =
    LazyLock::new(|| Selector::parse("div[data-lyrics-container=true]").unwrap());

#[derive(Default)]
struct Verse<'a> {
    title: &'a str,
    lyrics: Vec<String>,
}

#[derive(Template)]
#[template(path = "lyrics.html")]
struct LyricsTemplate<'a> {
    settings: Settings,
    verses: Vec<Verse<'a>>,
    path: &'a str,
    song: GeniusSong,
}

#[derive(Debug, Deserialize)]
pub struct LyricsQuery {
    id: Option<u32>,
}

#[get("/{path}-lyrics")]
pub async fn lyrics(req: HttpRequest, info: web::Query<LyricsQuery>) -> Result<impl Responder> {
    let document: Html;
    let song: GeniusSong;

    // The '-lyrics' bit of the path gets cut off since we match for it explicitly,
    // so we need to add it back here otherwise the path will be incorrect.
    let path = &format!(
        "{}-lyrics",
        req.match_info().query("path").trim_end_matches('?')
    );

    if let Some(id) = info.id {
        let responses = future::join(
            genius::get_text(genius::SubDomain::Root, path, None),
            genius::get_song(id),
        )
        .await;
        document = Html::parse_document(&responses.0?);
        song = responses.1?;
    } else {
        let lyric_page = genius::get_text(genius::SubDomain::Root, path, None).await?;
        document = Html::parse_document(&lyric_page);
        let id = get_song_id(&document)?;
        song = genius::get_song(id).await?;
    }

    let verses = scrape_lyrics(&document);

    Ok(template(
        &req,
        LyricsTemplate {
            settings: settings_from_req(&req),
            verses,
            path,
            song,
        },
    ))
}

fn get_song_id(document: &Html) -> crate::Result<u32> {
    Ok(document
        .select(&SONG_ID_SELECTOR)
        .next()
        .ok_or("Failed to find meta tag with song ID")?
        .value()
        .attr("content")
        .and_then(|content| content.strip_prefix("genius://songs/"))
        .ok_or("Failed to find content attribute")?
        .parse::<u32>()?)
}

fn scrape_lyrics(document: &Html) -> Vec<Verse> {
    let mut verses = Vec::new();
    let mut current_verse: Option<Verse> = None;
    let mut new_line = false;

    for child in document
        .select(&LYRIC_SELECTOR)
        .flat_map(|e| e.descendants())
    {
        match child.value() {
            Node::Element(e) if e.name() == "br" => {
                new_line = true;
            }
            Node::Text(text) => {
                let text: &str = text;
                let is_title = text.starts_with('[') && text.ends_with(']');
                if is_title {
                    if let Some(curr) = current_verse {
                        verses.push(curr);
                    }
                    current_verse = Some(Verse {
                        title: text,
                        lyrics: Vec::new(),
                    });
                } else {
                    let curr: &mut Verse = current_verse.get_or_insert_with(Verse::default);
                    let last = curr.lyrics.last_mut();
                    if new_line || last.is_none() {
                        curr.lyrics.push(text.to_owned());
                        new_line = false;
                    } else if let Some(lyric) = last {
                        lyric.push_str(text);
                    }
                }
            }
            _ => {}
        }
    }

    if let Some(curr) = current_verse {
        verses.push(curr);
    } else {
        verses.push(Verse {
            title: "",
            lyrics: vec!["This song has no lyrics.".to_owned()],
        })
    }

    verses
}
