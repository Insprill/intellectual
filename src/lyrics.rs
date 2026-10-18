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

    const CUT_THE_BRIDGE: &str =
        include_str!("../tests/fixtures/Linkin-park-cut-the-bridge-lyrics.html");
    const IN_THE_END: &str = include_str!("../tests/fixtures/Linkin-park-in-the-end-lyrics.html");
    const A_LINE_IN_THE_DUST: &str =
        include_str!("../tests/fixtures/Landmvrks-a-line-in-the-dust-lyrics.html");

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

    fn titles(verses: &[Verse]) -> Vec<&str> {
        verses.iter().map(|verse| verse.title.as_str()).collect()
    }

    /// The paragraph structure of each verse: `T` for a line of text, `.` for a blank line.
    fn shapes(verses: &[Verse]) -> Vec<String> {
        verses
            .iter()
            .map(|verse| {
                verse
                    .lyrics
                    .iter()
                    .map(|lyric| match lyric {
                        Lyric::Text(_) => 'T',
                        Lyric::Blank => '.',
                    })
                    .collect()
            })
            .collect()
    }

    fn lines(verse: &Verse) -> Vec<&TextLyric> {
        verse
            .lyrics
            .iter()
            .filter_map(|lyric| match lyric {
                Lyric::Text(text) => Some(text),
                Lyric::Blank => None,
            })
            .collect()
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
    fn cut_the_bridge() {
        let verses = parse(CUT_THE_BRIDGE);
        assert_eq!(
            titles(&verses),
            [
                "[Verse 1: Mike Shinoda]",
                "[Pre-Chorus: Emily Armstrong & Mike Shinoda]",
                "[Chorus: Emily Armstrong, All except Emily Armstrong]",
                "[Verse 2: Mike Shinoda]",
                "[Pre-Chorus: Emily Armstrong & Mike Shinoda]",
                "[Chorus: Emily Armstrong, All except Emily Armstrong]",
                "[Bridge: Mike Shinoda, Mike Shinoda & Emily Armstrong]",
                "[Chorus: Emily Armstrong, All]",
                "[Outro: All, All except Emily Armstrong]",
            ]
        );
        assert_eq!(
            shapes(&verses),
            [
                "TTTTTTTT",
                "TTTT",
                "TTTTTTTT",
                "TTTTTTTT",
                "TTTT",
                "TTTTTTTT",
                "TTTTTTTTTT",
                "TTTTTTTT",
                "T"
            ]
        );

        // An annotation spanning two lines applies to both of them.
        let verse_two = lines(&verses[3]);
        assert_eq!(verse_two[2].parts[0].annotation_id, Some(37514664));
        assert_eq!(verse_two[3].parts[0].annotation_id, Some(37514664));
        // Annotated and plain text on the same line are separate parts.
        assert_eq!(verse_two[0].parts.len(), 2);
        assert_eq!(verse_two[0].parts[0].annotation_id, Some(37834004));
        assert_eq!(verse_two[0].parts[1].annotation_id, None);
    }

    #[test]
    fn in_the_end() {
        let verses = parse(IN_THE_END);
        assert_eq!(
            titles(&verses),
            [
                "[Intro: Chester Bennington]",
                "[Verse 1: Mike Shinoda, Mike Shinoda & Chester Bennington]",
                "[Pre-Chorus: Mike Shinoda]",
                "[Chorus: Chester Bennington]",
                "[Verse 2: Mike Shinoda, Mike Shinoda & Chester Bennington]",
                "[Pre-Chorus: Mike Shinoda]",
                "[Chorus: Chester Bennington]",
                "[Bridge: Chester Bennington]",
                "[Chorus: Chester Bennington]",
            ]
        );
        assert_eq!(
            shapes(&verses),
            [
                "T",
                "TTTTTTTTTTTT",
                "TTTT",
                "TTTT",
                "TTTTTTTTTTTT",
                "TTTT",
                "TTTT",
                "TTTTTT",
                "TTTT"
            ]
        );
    }

    #[test]
    fn a_line_in_the_dust() {
        let verses = parse(A_LINE_IN_THE_DUST);
        assert_eq!(
            titles(&verses),
            [
                "[Verse 1: Florent Salfati]",
                "[Pre-Chorus: Florent Salfati]",
                "[Chorus: Florent Salfati]",
                "[Verse 2: Mat Welsh, Florent Salfati and Mat Welsh]",
                "[Pre-Chorus: Mat Welsh, Florent Salfati and Mat Welsh]",
                "[Chorus: Mat Welsh, Florent Salfati]",
                "[Breakdown: Mat Welsh, Florent Salfati, Florent Salfati and Mat Welsh]",
                "[Chorus: Mat Welsh, Florent Salfati and Mat Welsh]",
                "[Outro: Mat Welsh, Florent Salfati, Florent Salfati and Mat Welsh]",
            ]
        );
        // The second pre-chorus and the second half of the last chorus start a new container,
        // separated from the previous one by a sidebar block instead of a double <br>.
        assert_eq!(
            shapes(&verses),
            [
                "TTTT",
                "TTT",
                "TTT.TTTTT",
                "TTTTT",
                "TTT",
                "TTT.TTT",
                "TTTTT.TT",
                "TTTT.TTT",
                "TT"
            ]
        );
        // `<b>…for </b><b>real</b>` is a single part.
        let verse_two = lines(&verses[3]);
        assert_eq!(verse_two[0].parts.len(), 1);
        assert!(verse_two[0].parts[0].text.ends_with("for real"));
        // Every line consists of a single part since nothing here is annotated.
        assert!(
            verses
                .iter()
                .flat_map(lines)
                .all(|line| line.parts.len() == 1)
        );
    }
}
