use actix_web::{get, web, Responder, Result};
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
pub async fn artist(info: web::Query<ArtistQuery>) -> Result<impl Responder> {
    let response = genius::get_text(
        genius::SubDomain::Api,
        info.api_path.trim_start_matches('/'),
        None,
    )
    .await?;

    let res: GeniusArtistRequest = serde_json::from_str(&response)?;

    Ok(template(ArtistTemplate {
        artist: res.response.artist,
    }))
}
