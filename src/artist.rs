use crate::settings::{settings_from_req, Settings};
use crate::utils;
use actix_web::{get, HttpRequest, Responder, Result};
use askama::Template;
use lazy_regex::*;
use regex::Regex;

use crate::genius::{self, GeniusArtist};
use crate::genius::{GeniusArtistResponse, SortMode};
use crate::templates::template;

static GENIUS_IMAGE_URL: &str = "https://images.genius.com/";
static GENIUS_BASE_PATTERN: Lazy<Regex> = lazy_regex!(r#"https?://\w*.?genius\.com/"#);
static GENIUS_ALBUMS_PATTERN: Lazy<Regex> = lazy_regex!(r#"https?://\w*.?genius\.com/albums/"#);
static GENIUS_ARTIST_PATTERN: Lazy<Regex> = lazy_regex!(r#"https?://\w*.?genius\.com/"#);

#[derive(Template)]
#[template(path = "artist.html")]
struct ArtistTemplate {
    settings: Settings,
    artist: GeniusArtist,
}

const MAX_SONGS: u8 = 5;

#[get("/artists/{name}")]
pub async fn artist(req: HttpRequest) -> Result<impl Responder> {
    let mut artist = genius::extract_data::<GeniusArtistResponse>(req.path())
        .await?
        .artist;

    artist.popular_songs =
        Some(genius::get_artist_songs(artist.id, SortMode::Popularity, MAX_SONGS).await?);

    if let Some(description) = artist.description.as_mut() {
        description.html = rewrite_links(&description.html);
    }

    Ok(template(
        &req,
        ArtistTemplate {
            settings: settings_from_req(&req),
            artist,
        },
    ))
}

fn rewrite_links(html: &str) -> String {
    let html = html.replace(
        GENIUS_IMAGE_URL,
        &format!("/api/image?url={}", GENIUS_IMAGE_URL),
    ); // Images
    let html = GENIUS_ALBUMS_PATTERN.replace_all(&html, "/album?path=albums/"); // Albums
    let html = GENIUS_ARTIST_PATTERN.replace_all(&html, ""); // Artists
    let html = GENIUS_BASE_PATTERN.replace_all(&html, "/lyrics?path=/"); // Lyrics
    html.to_string()
}
