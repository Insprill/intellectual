use std::collections::{HashMap, HashSet};
use std::sync::LazyLock;

use actix_web::{get, web, HttpRequest, Responder, Result};
use askama::Template;
use futures::{future, StreamExt};

use futures::stream::FuturesUnordered;
use scraper::{Html, Node, Selector};
use serde::Deserialize;

use crate::genius::{self, GeniusAnnotationBody};
use crate::genius::{GeniusAnnotation, GeniusSong};
use crate::settings::{settings_from_req, Settings};
use crate::templates::template;
use crate::utils;

static SONG_ID_SELECTOR: LazyLock<Selector> =
    LazyLock::new(|| Selector::parse("meta[property='twitter:app:url:iphone']").unwrap());
static LYRIC_SELECTOR: LazyLock<Selector> =
    LazyLock::new(|| Selector::parse("div[data-lyrics-container=true]").unwrap());
// The summary that used to be in the page header is now part of the lyrics container in this div
static LYRIC_EXCLUDES_SELECTOR: LazyLock<Selector> =
    LazyLock::new(|| Selector::parse("div[data-exclude-from-selection]").unwrap());

#[derive(Default)]
struct Verse<'a> {
    title: &'a str,
    lyrics: Vec<Lyric>,
}

enum Lyric {
    Text(TextLyric),
    Blank,
}

struct TextLyric {
    parts: Vec<LyricPart>,
}

struct LyricPart {
    text: String,
    annotation: Option<GeniusAnnotation>,
}

#[derive(Template)]
#[template(path = "lyrics.html")]
struct LyricsTemplate<'a> {
    settings: Settings,
    verses: Vec<Verse<'a>>,
    annotations: Vec<GeniusAnnotation>,
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

    let (verses, annotations) = scrape_lyrics(&document).await?;

    Ok(template(
        &req,
        LyricsTemplate {
            settings: settings_from_req(&req),
            verses,
            annotations,
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

async fn scrape_lyrics(document: &Html) -> crate::Result<(Vec<Verse<'_>>, Vec<GeniusAnnotation>)> {
    let mut verses = Vec::new();
    let mut current_verse: Option<Verse> = None;
    let mut new_line = false;
    let mut curr_annotation: Option<GeniusAnnotation> = None;

    let excluded_elements: std::collections::HashSet<_> = document
        .select(&LYRIC_EXCLUDES_SELECTOR)
        .flat_map(|e| e.descendants())
        .map(|node| node.id())
        .collect();

    let mut annotations = HashSet::<&str>::new();

    for child in document
        .select(&LYRIC_SELECTOR)
        .flat_map(|e| e.descendants())
        .filter(|node| !excluded_elements.contains(&node.id()))
    {
        let curr: &mut Verse = current_verse.get_or_insert_with(Verse::default);
        match child.value() {
            Node::Element(e) if e.name() == "br" => {
                if new_line {
                    curr.lyrics.push(Lyric::Blank);
                }
                new_line = true;
            }
            Node::Element(e) if e.name() == "a" => {
                if let Some(href) = e.attr("href") {
                    if let Some((annotation_id, _)) = href.trim_start_matches('/').split_once('/') {
                        curr_annotation = Option::Some(GeniusAnnotation {
                            id: annotation_id.parse::<i32>()?,
                            body: GeniusAnnotationBody {
                                html: String::new(),
                            },
                        });
                        annotations.insert(annotation_id);
                    }
                }
            }
            Node::Text(text) => {
                let text: &str = text;
                let is_title = text.starts_with('[') && text.ends_with(']');
                if is_title {
                    new_line = false;
                    if let Some(mut curr) = current_verse {
                        // Remove trailing blank lines
                        while matches!(curr.lyrics.last(), Some(s) if matches!(s, Lyric::Blank)) {
                            curr.lyrics.pop();
                        }
                        verses.push(curr);
                    }
                    current_verse = Some(Verse {
                        title: text,
                        lyrics: Vec::new(),
                    });
                } else {
                    let last = curr.lyrics.last_mut();
                    if new_line || last.is_none() {
                        curr.lyrics.push(Lyric::Text(TextLyric {
                            parts: vec![LyricPart {
                                text: text.to_owned(),
                                annotation: curr_annotation,
                            }],
                        }));
                        new_line = false;
                    } else if let Some(lyric) = last {
                        if let Lyric::Text(text_lyric) = lyric {
                            text_lyric.parts.push(LyricPart {
                                text: text.to_string(),
                                annotation: curr_annotation,
                            });
                        }
                    }
                    curr_annotation = None;
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
            lyrics: vec![Lyric::Text(TextLyric {
                parts: vec![LyricPart {
                    text: "This song has no lyrics.".to_owned(),
                    annotation: None,
                }],
            })],
        })
    }

    let annotations: HashMap<i32, GeniusAnnotation> = verses
        .iter()
        .flat_map(|v| {
            v.lyrics
                .iter()
                .filter_map(|l| match l {
                    Lyric::Text(tl) => Some(tl),
                    _ => None,
                })
                .flat_map(|text_lyric| {
                    text_lyric
                        .parts
                        .iter()
                        .filter_map(|p| p.annotation.as_ref().map(|a| a.id))
                })
        })
        .map(|id| genius::get_annotation(id))
        .collect::<FuturesUnordered<_>>()
        .collect::<Vec<_>>()
        .await
        .into_iter()
        .filter_map(|f| f.ok())
        .map(|a| (a.id, a))
        .collect();

    for v in &mut verses {
        for l in &mut v.lyrics {
            if let Lyric::Text(tl) = l {
                for p in &mut tl.parts {
                    if let Some(a) = &p.annotation {
                        p.annotation = annotations.get(&a.id).cloned();
                    }
                }
            }
        }
    }
    
    Ok((verses, annotations.into_values().collect()))
}
