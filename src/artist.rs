use crate::settings::{settings_from_req, Settings};
use crate::utils;
use actix_web::{get, HttpRequest, Responder, Result};
use askama::Template;

use crate::genius::{self, GeniusArtist};
use crate::genius::{GeniusArtistResponse, SortMode};
use crate::templates::template;

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

    Ok(template(ArtistTemplate {
        settings: settings_from_req(&req),
        artist,
    }))
}
