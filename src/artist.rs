use crate::utils;
use actix_web::{get, web, Responder, Result};
use askama::Template;
use serde::Deserialize;

use crate::genius::{self, GeniusSong, GeniusSongsRequest};
use crate::genius::{GeniusArtist, GeniusArtistRequest};
use crate::templates::template;

#[derive(Template)]
#[template(path = "artist.html")]
struct ArtistTemplate {
    artist: GeniusArtist,
}

#[derive(Debug, Deserialize)]
pub struct ArtistQuery {
    api_path: String,
}

#[get("/artist")]
pub async fn artist(info: web::Query<ArtistQuery>) -> Result<impl Responder> {
    let response = genius::get_text(
        genius::SubDomain::Api,
        info.api_path.trim_start_matches('/'),
        None,
    )
    .await?;

    let mut artist: GeniusArtist = serde_json::from_str::<GeniusArtistRequest>(&response)?
        .response
        .artist;

    let response = genius::get_text(
        genius::SubDomain::Api,
        &format!("{}/songs", info.api_path.trim_start_matches('/')),
        Some(vec![("sort", "popularity"), ("per_page", "5")]),
    )
    .await?;

    let songs: Vec<GeniusSong> = serde_json::from_str::<GeniusSongsRequest>(&response)?
        .response
        .songs;
    artist.popular_songs = Some(songs);

    Ok(template(ArtistTemplate { artist }))
}
