use std::collections::HashSet;
use std::sync::LazyLock;

use actix_web::{HttpRequest, Responder, Result, get, web};
use askama::Template;
use futures::{StreamExt, future};

use futures::stream::FuturesUnordered;
use scraper::node::Element;
use scraper::{ElementRef, Html, Node, Selector};
use serde::Deserialize;

use crate::genius::{self, GeniusReferentResponse, GeniusSong};
use crate::settings::{Settings, settings_from_req};
use crate::templates::template;
use crate::utils;

static SONG_ID_SELECTOR: LazyLock<Selector> =
    LazyLock::new(|| Selector::parse("meta[property='twitter:app:url:iphone']").unwrap());
static LYRIC_SELECTOR: LazyLock<Selector> =
    LazyLock::new(|| Selector::parse("div[data-lyrics-container=true]").unwrap());

#[derive(Default)]
struct Verse {
    title: String,
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
    /// The ID of the Genius referent (annotation) this part of the line is linked to.
    annotation_id: Option<i32>,
}

#[derive(Default, Clone)]
struct Annotation {
    pub id: i32,
    pub quote: String,
    pub body: String,
    pub votes: i32,
}

impl From<&GeniusReferentResponse> for Annotation {
    fn from(value: &GeniusReferentResponse) -> Self {
        Annotation {
            id: value.referent.id,
            quote: value.referent.fragment.clone(),
            body: value.referent.annotations[0].body.html.clone(),
            votes: value.referent.annotations[0].votes_total,
        }
    }
}

#[derive(Template)]
#[template(path = "lyrics.html")]
struct LyricsTemplate<'a> {
    settings: Settings,
    verses: Vec<Verse>,
    annotations: Vec<Annotation>,
    path: &'a str,
    song: GeniusSong,
}

#[derive(Debug, Deserialize)]
pub struct LyricsQuery {
    id: Option<u32>,
}

// Non-song texts (speeches, poems, etc.) end in '-annotated' instead of '-lyrics'.
#[get("/{path:[^/]+-(?:lyrics|annotated)}")]
pub async fn lyrics(req: HttpRequest, info: web::Query<LyricsQuery>) -> Result<impl Responder> {
    let document: Html;
    let song: GeniusSong;

    let path = req.match_info().query("path").trim_end_matches('?');

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

    let (verses, annotations) = scrape_lyrics(&document).await;

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

async fn scrape_lyrics(document: &Html) -> (Vec<Verse>, Vec<Annotation>) {
    let verses = parse_lyrics(document);

    let annotation_ids: HashSet<i32> = verses
        .iter()
        .flat_map(|verse| &verse.lyrics)
        .filter_map(|lyric| match lyric {
            Lyric::Text(text) => Some(&text.parts),
            Lyric::Blank => None,
        })
        .flatten()
        .filter_map(|part| part.annotation_id)
        .collect();

    let annotations = annotation_ids
        .into_iter()
        .map(genius::get_annotation)
        .collect::<FuturesUnordered<_>>()
        .collect::<Vec<_>>()
        .await
        .into_iter()
        .filter_map(|res| res.ok())
        .filter(|res| !res.referent.annotations.is_empty())
        .map(|res| Annotation::from(&res))
        .collect();

    (verses, annotations)
}

/// Turns the lyrics containers of a Genius song page into verses.
///
/// Genius splits the lyrics over several `data-lyrics-container` divs with ads and sidebars in between
/// them. Inside a container, lines are separated by `<br>`, paragraphs by two of them, and section
/// headings are lines wrapped in square brackets. Any of those may be split over inline elements
/// (`<i>`, `<b>`, annotation links), so lines are always judged by their combined text.
fn parse_lyrics(document: &Html) -> Vec<Verse> {
    let mut builder = LyricsBuilder::default();
    for container in document.select(&LYRIC_SELECTOR) {
        builder.block_break();
        collect_lyrics(container, None, &mut builder);
    }

    let mut verses = builder.finish();
    if verses.is_empty() {
        verses.push(Verse {
            title: String::new(),
            lyrics: vec![Lyric::Text(TextLyric {
                parts: vec![LyricPart {
                    text: "This song has no lyrics.".to_owned(),
                    annotation_id: None,
                }],
            })],
        });
    }
    verses
}

fn collect_lyrics(element: ElementRef, annotation_id: Option<i32>, builder: &mut LyricsBuilder) {
    for child in element.children() {
        match child.value() {
            Node::Text(text) => builder.text(text, annotation_id),
            Node::Element(e) if e.name() == "br" => builder.line_break(),
            // The song summary and contributor information sit inside the first container in this div.
            Node::Element(e) if e.attr("data-exclude-from-selection").is_some() => {
                builder.block_break();
            }
            Node::Element(e) => {
                let annotation_id = annotation_id.or_else(|| get_annotation_id(e));
                if let Some(child) = ElementRef::wrap(child) {
                    collect_lyrics(child, annotation_id, builder);
                }
            }
            _ => {}
        }
    }
}

/// Annotated lyrics are wrapped in a link to the referent, e.g. `/12345/Artist-song/Fragment`.
fn get_annotation_id(element: &Element) -> Option<i32> {
    if element.name() != "a" {
        return None;
    }
    let path = element.attr("href")?.trim_start_matches('/');
    path.split_once('/')?.0.parse().ok()
}

#[derive(Default)]
struct LyricsBuilder {
    verses: Vec<Verse>,
    verse: Verse,
    /// The parts of the line currently being read.
    line: Vec<LyricPart>,
}

impl LyricsBuilder {
    fn text(&mut self, text: &str, annotation_id: Option<i32>) {
        if text.is_empty() {
            return;
        }
        // Text split over inline elements, like `<i>M</i><i>ike</i>`, belongs to the same part so
        // the template won't put whitespace between the pieces.
        match self.line.last_mut() {
            Some(last) if last.annotation_id == annotation_id => last.text.push_str(text),
            _ => self.line.push(LyricPart {
                text: text.to_owned(),
                annotation_id,
            }),
        }
    }

    fn line_break(&mut self) {
        let parts = std::mem::take(&mut self.line);
        let text = parts
            .iter()
            .map(|part| part.text.as_str())
            .collect::<String>();
        let text = text.trim();
        if text.is_empty() {
            self.paragraph_break();
        } else if text.starts_with('[') && text.ends_with(']') {
            self.finish_verse();
            self.verse.title = text.to_owned();
        } else {
            self.verse.lyrics.push(Lyric::Text(TextLyric { parts }));
        }
    }

    /// Something that isn't lyrics (another container, an ad, the song summary) ended or started
    /// here, which cuts the current line short and separates paragraphs on either side of it.
    fn block_break(&mut self) {
        self.line_break();
        self.paragraph_break();
    }

    fn paragraph_break(&mut self) {
        if matches!(self.verse.lyrics.last(), Some(Lyric::Text(_))) {
            self.verse.lyrics.push(Lyric::Blank);
        }
    }

    fn finish_verse(&mut self) {
        let mut verse = std::mem::take(&mut self.verse);
        while matches!(verse.lyrics.last(), Some(Lyric::Blank)) {
            verse.lyrics.pop();
        }
        if !verse.title.is_empty() || !verse.lyrics.is_empty() {
            self.verses.push(verse);
        }
    }

    fn finish(mut self) -> Vec<Verse> {
        self.line_break();
        self.finish_verse();
        self.verses
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The lyrics part of a Genius song page, laid out the way Genius sends it.
    const GENIUS_PAGE: &str = concat!(
        "<html><body>",
        "<div data-lyrics-container=\"true\" class=\"Lyrics__Container-sc-12f9e8d3-1 jhVhlB\">",
        "<div data-exclude-from-selection=\"true\"><button><span>3 Contributors</span></button>",
        "<h2 font-weight=\"light\">Lorem Ipsum Lyrics</h2></div>",
        "[Verse 1: <b>Lorem</b>, <i>Ipsum &amp; Dolor</i>]<br/>",
        "Lorem ipsum <i>dolor</i> sit amet<br/>",
        "<a href=\"/111/Lorem-ipsum/Consectetur-adipiscing-elit\" data-ignore-on-click-outside=\"true\">",
        "<span>Consectetur adipiscing elit</span></a>",
        "<span tabindex=\"0\"></span><span><span tabindex=\"0\"></span></span><br/>",
        "<a href=\"/222/Lorem-ipsum/Sed-do-eiusmod\" data-ignore-on-click-outside=\"true\">",
        "<span>Sed do eiusmod</span></a>",
        "<span tabindex=\"0\"></span><span><span tabindex=\"0\"></span></span> tempor incididunt<br/>",
        "<b>Ut labore et </b><b>dolore</b><br/><br/>",
        "Magna aliqua ut enim<br/>",
        "<a href=\"/333/Lorem-ipsum/Ad-minim-veniam-quis-nostrud\" data-ignore-on-click-outside=\"true\">",
        "<span>Ad minim <i>veniam</i><br/>Quis nostrud</span></a>",
        "<span tabindex=\"0\"></span><span><span tabindex=\"0\"></span></span><br/>",
        "</div>",
        "<div class=\"RightSidebar__Container-sc-eb801a17-0\">",
        "<div id=\"div-gpt-ad-desktop_song_lyrics_sidebar-desktop_song_lyrics_sidebar-1\"></div>",
        "<aside><div>You might also like</div></aside></div>",
        "<div data-lyrics-container=\"true\" class=\"Lyrics__Container-sc-12f9e8d3-1 jhVhlB\"></div>",
        "<div data-exclude-from-selection=\"true\">",
        "<div id=\"div-gpt-ad-desktop_song_lyrics_inread-desktop_song_lyrics_inread-1\"></div></div>",
        "<div data-lyrics-container=\"true\" class=\"Lyrics__Container-sc-12f9e8d3-1 jhVhlB\">",
        "Exercitation ullamco<br/>Laboris nisi ut<br/><br/>",
        "<a href=\"/444/Lorem-ipsum/Chorus-lorem\" data-ignore-on-click-outside=\"true\">",
        "<span>[Chorus: Lorem]</span></a>",
        "<span tabindex=\"0\"></span><span><span tabindex=\"0\"></span></span><br/>",
        "Duis aute irure<br/>",
        "<i>Dolor in reprehenderit</i>",
        "</div>",
        "</body></html>",
    );

    fn parse(html: &str) -> Vec<Verse> {
        parse_lyrics(&Html::parse_document(html))
    }

    fn container(body: &str) -> String {
        format!("<html><body><div data-lyrics-container=\"true\">{body}</div></body></html>")
    }

    /// Renders verses as text: the title on its own line, then one line per lyric, where each
    /// part is wrapped in `|` and suffixed with `@id` when annotated, and blank lines are empty.
    fn render(verses: &[Verse]) -> String {
        let mut out = String::new();
        for verse in verses {
            out.push_str(&verse.title);
            out.push('\n');
            for lyric in &verse.lyrics {
                if let Lyric::Text(text) = lyric {
                    for part in &text.parts {
                        out.push('|');
                        out.push_str(&part.text);
                        if let Some(id) = part.annotation_id {
                            out.push_str(&format!("@{id}"));
                        }
                        out.push('|');
                    }
                }
                out.push('\n');
            }
        }
        out
    }

    #[test]
    fn text_split_over_inline_elements() {
        let verses = parse(&container(
            "[Ch<i>or</i>us: <b>S</b><b>omeone</b>]<br><i>F</i><i>irst</i> line<br>Se<b>cond</b> <i>line</i>",
        ));
        assert_eq!(
            render(&verses),
            "[Chorus: Someone]\n|First line|\n|Second line|\n"
        );
    }

    #[test]
    fn annotations() {
        let verses = parse(&container(concat!(
            "[Verse 1]<br>",
            "<a href=\"/123/Artist-song/First-line\"><span>First line</span></a><span tabindex=\"0\"></span><br>",
            "Second <a href=\"/456/Artist-song/Line\"><span>li</span></a><span tabindex=\"0\"></span>ne<br>",
            "<a href=\"/789/Artist-song/Third-line-fourth-line\"><span>Third line<br>Fourth line</span></a>",
            "<span tabindex=\"0\"></span><br>",
            "<a href=\"https://example.com/not-an-annotation\">Fifth line</a><br>",
        )));
        assert_eq!(
            render(&verses),
            "[Verse 1]\n|First line@123|\n|Second ||li@456||ne|\n|Third line@789|\n|Fourth line@789|\n|Fifth line|\n"
        );
    }

    #[test]
    fn template_puts_no_whitespace_between_parts() {
        let song: GeniusSong = serde_json::from_str(
            r#"{
                "id": 1, "title": "Song", "path": "/Artist-song-lyrics",
                "header_image_url": "https://example.com/header.png",
                "release_date_for_display": null,
                "song_art_image_thumbnail_url": "https://example.com/thumb.png",
                "album": null, "stats": { "pageviews": null },
                "primary_artist": {
                    "id": 1, "name": "Artist", "alternate_names": null,
                    "image_url": "https://example.com/artist.png",
                    "url": "https://genius.com/artists/Artist",
                    "description": null, "popular_songs": null, "facebook_name": null
                }
            }"#,
        )
        .unwrap();
        let verses = parse(&container(
            "[Verse 1]<br><i>F</i><i>irst</i> <a href=\"/123/Artist-song/Li\"><span>li</span></a>ne",
        ));
        let page = LyricsTemplate {
            settings: Settings::default(),
            verses,
            annotations: Vec::new(),
            path: "Artist-song-lyrics",
            song,
        }
        .render()
        .unwrap();
        assert!(
            page.contains("<h2 class=\"song-lyric\">[Verse 1]</h2>"),
            "{page}"
        );
        assert!(
            page.contains(concat!(
                "<p class=\"song-lyric\">First ",
                "<span><a class=\"annotation-link\" href=\"#annotation-123\">li</a></span>ne</p>"
            )),
            "{page}"
        );
    }

    #[test]
    fn genius_page() {
        // The sidebar and ad between the containers split the first verse into two paragraphs,
        // and the annotated heading still starts a new verse.
        assert_eq!(
            render(&parse(GENIUS_PAGE)),
            concat!(
                "[Verse 1: Lorem, Ipsum & Dolor]\n",
                "|Lorem ipsum dolor sit amet|\n",
                "|Consectetur adipiscing elit@111|\n",
                "|Sed do eiusmod@222|| tempor incididunt|\n",
                "|Ut labore et dolore|\n",
                "\n",
                "|Magna aliqua ut enim|\n",
                "|Ad minim veniam@333|\n",
                "|Quis nostrud@333|\n",
                "\n",
                "|Exercitation ullamco|\n",
                "|Laboris nisi ut|\n",
                "[Chorus: Lorem]\n",
                "|Duis aute irure|\n",
                "|Dolor in reprehenderit|\n",
            )
        );
    }
}
