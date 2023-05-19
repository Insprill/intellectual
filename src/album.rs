use crate::utils;
use actix_web::{get, web, Responder, Result};
use askama::Template;
use serde::Deserialize;

use crate::genius::GeniusAlbum;
use crate::genius::GeniusApi;
use crate::templates::template;

#[derive(Template)]
#[template(path = "album.html")]
struct AlbumTemplate {
    album: GeniusAlbum,
}

#[derive(Debug, Deserialize)]
pub struct AlbumQuery {
    id: u32,
}

#[get("/album")]
pub async fn album(info: web::Query<AlbumQuery>) -> Result<impl Responder> {
    let mut album = GeniusApi::global().get_album(info.id).await?;

    album.tracks = Some(GeniusApi::global().get_album_tracks(info.id).await?);

    Ok(template(AlbumTemplate { album }))
}
