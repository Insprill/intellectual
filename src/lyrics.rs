use std::collections::{HashMap, HashSet};
use std::sync::LazyLock;

use actix_web::{HttpRequest, Responder, Result, get, web};
use askama::Template;
use futures::future;
use log::warn;
use scraper::{Html, Node, Selector};
use serde::Deserialize;

use crate::genius::{self, GeniusReferent, GeniusSong};
use crate::settings::{Settings, settings_from_req};
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
    referent: Option<Referent>,
}

#[derive(Clone, Copy)]
struct Referent {
    id: u32,
    // Repeated sections share a referent, so the id alone can't anchor the clicked line.
    occurrence: usize,
    // Only the first part of each occurrence carries the anchor.
    anchor: bool,
}

impl Referent {
    // The first occurrence keeps Genius's own `note-{id}`, so their links open it.
    fn note_id(&self) -> String {
        match self.occurrence {
            1 => format!("note-{}", self.id),
            n => format!("note-{}-{n}", self.id),
        }
    }
}

struct Annotation {
    pub quote: String,
    pub body: String,
    pub votes: i32,
}

impl Annotation {
    fn from_referent(referent: GeniusReferent) -> Option<(u32, Self)> {
        let annotation = referent.annotations.into_iter().next()?;
        Some((
            referent.id,
            Annotation {
                quote: referent.fragment,
                body: annotation.body.html,
                votes: annotation.votes_total,
            },
        ))
    }
}

#[derive(Template)]
#[template(path = "lyrics.html")]
struct LyricsTemplate<'a> {
    settings: Settings,
    verses: Vec<Verse<'a>>,
    // Keyed by referent id. Empty unless the page was opened through an annotation link.
    annotations: HashMap<u32, Annotation>,
    path: &'a str,
    song: GeniusSong,
}

#[derive(Debug, Deserialize)]
pub struct LyricsQuery {
    id: Option<u32>,
    // Same format Genius redirects annotation links to (`?referent_id={id}#note-{id}`).
    // Its presence is what loads annotations; the fragment picks the one to show.
    referent_id: Option<u32>,
}

// Non-song texts (speeches, poems, etc.) end in '-annotated' instead of '-lyrics'.
#[get("/{path:[^/]+-(?:lyrics|annotated)}")]
pub async fn lyrics(req: HttpRequest, info: web::Query<LyricsQuery>) -> Result<impl Responder> {
    let document: Html;
    let song: GeniusSong;
    let annotations: HashMap<u32, Annotation>;

    let path = req.match_info().query("path").trim_end_matches('?');

    if let Some(id) = info.id {
        let responses = future::join3(
            genius::get_text(genius::SubDomain::Root, path, None),
            genius::get_song(id),
            get_annotations(id, info.referent_id),
        )
        .await;
        document = Html::parse_document(&responses.0?);
        song = responses.1?;
        annotations = responses.2;
    } else {
        let lyric_page = genius::get_text(genius::SubDomain::Root, path, None).await?;
        document = Html::parse_document(&lyric_page);
        let id = get_song_id(&document)?;
        let responses =
            future::join(genius::get_song(id), get_annotations(id, info.referent_id)).await;
        song = responses.0?;
        annotations = responses.1;
    }

    let verses = scrape_lyrics(&document);

    Ok(template(LyricsTemplate {
        settings: settings_from_req(&req),
        verses,
        annotations,
        path,
        song,
    }))
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

// Genius caps referent pages at 50.
const REFERENTS_PER_PAGE: u8 = 50;
const MAX_REFERENT_PAGES: u8 = 5;

// Failed annotations shouldn't take the lyrics down with them.
async fn get_annotations(song_id: u32, referent_id: Option<u32>) -> HashMap<u32, Annotation> {
    let mut annotations = HashMap::new();
    let Some(referent_id) = referent_id else {
        return annotations;
    };

    for page in 1..=MAX_REFERENT_PAGES {
        let referents = match genius::get_song_referents(song_id, page, REFERENTS_PER_PAGE).await {
            Ok(referents) => referents,
            Err(err) => {
                warn!("Failed to fetch annotations for song {song_id} (page {page}): {err}");
                break;
            }
        };
        let count = referents.len();
        annotations.extend(referents.into_iter().filter_map(Annotation::from_referent));
        // The response's `next_page` is always null, so a short page is the only end marker.
        if count < REFERENTS_PER_PAGE as usize {
            break;
        }
    }

    // The list omits verified annotations, so the clicked one may still be missing.
    if !annotations.contains_key(&referent_id) {
        match genius::get_referent(referent_id).await {
            Ok(referent) => annotations.extend(Annotation::from_referent(referent)),
            Err(err) => warn!("Failed to fetch annotation for referent {referent_id}: {err}"),
        }
    }

    annotations
}

fn scrape_lyrics(document: &Html) -> Vec<Verse<'_>> {
    let mut verses = Vec::new();
    let mut current_verse: Option<Verse> = None;
    let mut new_line = false;
    let mut curr_referent: Option<Referent> = None;
    let mut occurrences = HashMap::<u32, usize>::new();

    let excluded_elements: HashSet<_> = document
        .select(&LYRIC_EXCLUDES_SELECTOR)
        .flat_map(|e| e.descendants())
        .map(|node| node.id())
        .collect();

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
            // Annotated lyrics link to `/{referent_id}/{song-slug}/{lyric-slug}`.
            Node::Element(e) if e.name() == "a" => {
                if let Some(href) = e.attr("href")
                    && let Some((referent_id, _)) = href.trim_start_matches('/').split_once('/')
                    && let Ok(referent_id) = referent_id.parse::<u32>()
                {
                    let occurrence = occurrences.entry(referent_id).or_default();
                    *occurrence += 1;
                    curr_referent = Some(Referent {
                        id: referent_id,
                        occurrence: *occurrence,
                        anchor: true,
                    });
                }
            }
            // Empty span with `tabindex="0"` always follows annotations.
            Node::Element(e)
                if e.name() == "span"
                    && e.attr("tabindex") == Some("0")
                    && curr_referent.is_some() =>
            {
                curr_referent = None;
            }
            Node::Text(text) => {
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
                    let part = LyricPart {
                        text: text.to_string(),
                        referent: curr_referent,
                    };
                    if let Some(referent) = curr_referent.as_mut() {
                        referent.anchor = false;
                    }
                    let last = curr.lyrics.last_mut();
                    if new_line || last.is_none() {
                        curr.lyrics
                            .push(Lyric::Text(TextLyric { parts: vec![part] }));
                        new_line = false;
                    } else if let Some(Lyric::Text(text_lyric)) = last {
                        text_lyric.parts.push(part);
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
            lyrics: vec![Lyric::Text(TextLyric {
                parts: vec![LyricPart {
                    text: "This song has no lyrics.".to_owned(),
                    referent: None,
                }],
            })],
        })
    }

    verses
}

#[cfg(test)]
mod tests {
    use super::*;

    // Genius serves the lyrics container minified, without whitespace between elements.
    const LYRICS: &str = concat!(
        r#"<div data-lyrics-container="true">"#,
        r#"<div data-exclude-from-selection="true"><a href="/artists/Some-artist">Artist</a> summary</div>"#,
        "[Intro]<br>",
        r#"<a href="/101/Some-song/First-line"><span>First line<br>of the referent</span></a><span tabindex="0"></span><br>"#,
        "Plain line<br><br>",
        "[Chorus]<br>",
        r#"Plain start, <a href="/202/Some-song/Annotated-end">annotated end</a><span tabindex="0"></span><br>"#,
        r#"<a href="/202/Some-song/Annotated-end">Repeated</a><span tabindex="0"></span>"#,
        "</div>"
    );

    fn parts<'a>(verse: &'a Verse, line: usize) -> &'a [LyricPart] {
        match &verse.lyrics[line] {
            Lyric::Text(text) => &text.parts,
            Lyric::Blank => panic!("line {line} is blank"),
        }
    }

    fn referent(part: &LyricPart) -> Option<(u32, usize, bool)> {
        part.referent
            .map(|referent| (referent.id, referent.occurrence, referent.anchor))
    }

    #[test]
    fn scrape_lyrics_keeps_referent_ids_without_fetching() {
        let document = Html::parse_document(LYRICS);
        let verses = scrape_lyrics(&document);
        let verse = |title| verses.iter().find(|v| v.title == title).unwrap();

        let intro = verse("[Intro]");
        assert_eq!(intro.lyrics.len(), 3);
        assert_eq!(parts(intro, 0)[0].text, "First line");
        assert_eq!(referent(&parts(intro, 0)[0]), Some((101, 1, true)));
        assert_eq!(parts(intro, 1)[0].text, "of the referent");
        assert_eq!(referent(&parts(intro, 1)[0]), Some((101, 1, false)));
        assert_eq!(referent(&parts(intro, 2)[0]), None);

        let line = parts(verse("[Chorus]"), 0);
        assert_eq!(referent(&line[0]), None);
        assert_eq!(line[1].text, "annotated end");
        assert_eq!(referent(&line[1]), Some((202, 1, true)));

        // A repeated section links to the same referent again; it needs its own anchor.
        let repeated = &parts(verse("[Chorus]"), 1)[0];
        assert_eq!(referent(repeated), Some((202, 2, true)));
        assert_eq!(line[1].referent.unwrap().note_id(), "note-202");
        assert_eq!(repeated.referent.unwrap().note_id(), "note-202-2");
    }
}
