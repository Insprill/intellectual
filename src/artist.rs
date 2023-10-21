use crate::settings::{Settings, settings_from_req};
use crate::utils;
use actix_web::{get, web, Responder, Result, HttpRequest};
use askama::Template;
use serde::Deserialize;

use crate::genius::{GeniusApi, GeniusArtist};
use crate::genius::{GeniusArtistResponse, SortMode};
use crate::templates::template;

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

    Ok(template(ArtistTemplate { 
        settings: settings_from_req(&req),
        artist 
    }))
}
