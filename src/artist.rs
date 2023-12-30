use crate::settings::{settings_from_req, Settings};
use crate::utils;
use actix_web::{get, web, HttpRequest, Responder, Result};
use askama::Template;
use lazy_regex::*;
use regex::Regex;
use serde::Deserialize;

use crate::genius::{GeniusApi, GeniusArtist};
use crate::genius::{GeniusArtistResponse, SortMode};
use crate::templates::template;

static GENIUS_IMAGE_URL: &str = "https://images.genius.com/";
static GENIUS_BASE_PATTERN: Lazy<Regex> = lazy_regex!(r#"https?://\w*.?genius\.com/"#);
static GENIUS_ALBUMS_PATTERN: Lazy<Regex> = lazy_regex!(r#"https?://\w*.?genius\.com/albums/"#);
static GENIUS_ARTIST_PATTERN: Lazy<Regex> = lazy_regex!(r#"https?://\w*.?genius\.com/artists/"#);

#[derive(Template)]
#[template(path = "artist.html")]
struct ArtistTemplate {
    settings: Settings,
    artist: GeniusArtist,
}

#[derive(Debug, Deserialize)]
pub struct ArtistQuery {
    path: String,
}

const MAX_SONGS: u8 = 5;

#[get("/artist")]
pub async fn artist(req: HttpRequest, info: web::Query<ArtistQuery>) -> Result<impl Responder> {
    let artist_res = GeniusApi::global()
        .extract_data::<GeniusArtistResponse>(&utils::ensure_path_prefix("artists", &info.path))
        .await?;
    let mut artist = artist_res.artist;

    artist.popular_songs = Some(
        GeniusApi::global()
            .get_artist_songs(artist.id, SortMode::Popularity, MAX_SONGS)
            .await?,
    );

    if let Some(description) = artist.description.as_mut() {
        description.html = rewrite_links(&description.html);
    }

    Ok(template(ArtistTemplate {
        settings: settings_from_req(&req),
        artist,
    }))
}

fn rewrite_links(html: &str) -> String {
    let html = html.replace(
        GENIUS_IMAGE_URL,
        &format!("/api/image?url={}", GENIUS_IMAGE_URL),
    ); // Images
    let html = GENIUS_ALBUMS_PATTERN.replace_all(&html, "/album?path=albums/"); // Albums
    let html = GENIUS_ARTIST_PATTERN.replace_all(&html, "/artist?path=artists/"); // Artists
    let html = GENIUS_BASE_PATTERN.replace_all(&html, "/lyrics?path=/"); // Lyrics
    html.to_string()
}
