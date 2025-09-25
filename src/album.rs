use crate::genius::{self, GeniusAlbumResponse};
use crate::settings::{settings_from_req, Settings};
use crate::utils;
use actix_web::HttpRequest;
use actix_web::{get, Responder, Result};
use askama::Template;

use crate::genius::GeniusAlbum;
use crate::templates::template;

#[derive(Template)]
#[template(path = "album.html")]
struct AlbumTemplate {
    settings: Settings,
    album: GeniusAlbum,
}

#[get("/albums/{name:.*}")]
pub async fn album(req: HttpRequest) -> Result<impl Responder> {
    let mut album = genius::extract_data::<GeniusAlbumResponse>(req.path())
        .await?
        .album;

    album.tracks = Some(genius::get_album_tracks(album.id).await?);

    Ok(template(AlbumTemplate {
        settings: settings_from_req(&req),
        album,
    }))
}
