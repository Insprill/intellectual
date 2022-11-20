use actix_web::{get, web, Responder};
use askama::Template;
use serde::Deserialize;

use crate::genius;
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
pub async fn artist(info: web::Query<ArtistQuery>) -> impl Responder {
    let response = genius::text(
        genius::SubDomain::Api,
        info.api_path.trim_start_matches('/'),
        None,
    )
    .await;
    let api: GeniusArtistRequest = serde_json::from_str(&response).unwrap();
    template(ArtistTemplate {
        artist: api.response.artist,
    })
}
