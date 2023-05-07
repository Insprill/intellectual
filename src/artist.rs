use crate::utils;
use actix_web::{get, web, Responder, Result};
use askama::Template;
use serde::Deserialize;

use crate::genius::SortMode;
use crate::genius::{GeniusApi, GeniusArtist};
use crate::templates::template;

#[derive(Template)]
#[template(path = "artist.html")]
struct ArtistTemplate {
    artist: GeniusArtist,
}

#[derive(Debug, Deserialize)]
pub struct ArtistQuery {
    id: u32,
}

const MAX_SONGS: u32 = 5;

#[get("/artist")]
pub async fn artist(info: web::Query<ArtistQuery>) -> Result<impl Responder> {
    let mut artist: GeniusArtist = GeniusApi::global().get_artist(info.id).await?;

    artist.popular_songs = Some(
        GeniusApi::global()
            .get_artist_songs(info.id, SortMode::Popularity, MAX_SONGS)
            .await?,
    );

    Ok(template(ArtistTemplate { artist }))
}
