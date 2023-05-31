use crate::genius::GeniusAlbumResponse;
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
    path: String,
}

#[get("/album")]
pub async fn album(info: web::Query<AlbumQuery>) -> Result<impl Responder> {
    let album_res = GeniusApi::global()
        .extract_data::<GeniusAlbumResponse>(&info.path)
        .await?;
    let mut album = album_res.album;

    album.tracks = Some(GeniusApi::global().get_album_tracks(album.id).await?);

    Ok(template(AlbumTemplate { album }))
}
