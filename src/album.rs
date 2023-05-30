use crate::genius;
use crate::genius::GeniusAlbumResponse;
use crate::utils;
use actix_web::{get, web, Responder, Result};
use askama::Template;
use once_cell::sync::Lazy;
use scraper::Html;
use scraper::Selector;
use serde::Deserialize;

use crate::genius::GeniusAlbum;
use crate::genius::GeniusApi;
use crate::templates::template;

static ALBUM_INFO_SELECTOR: Lazy<Selector> =
    Lazy::new(|| Selector::parse("meta[content]").unwrap());

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
    let mut album: GeniusAlbum;

    let lyric_page = GeniusApi::global()
        .get_text(genius::SubDomain::Root, &info.path, None)
        .await?;
    let document = Html::parse_document(&lyric_page);
    album = get_album(&document)?;

    album.tracks = Some(GeniusApi::global().get_album_tracks(album.id).await?);

    Ok(template(AlbumTemplate { album }))
}

fn get_album(document: &Html) -> crate::Result<GeniusAlbum> {
    Ok(document
        .select(&ALBUM_INFO_SELECTOR)
        .map(|element| element.value().attr("content").unwrap()) // Selector only matches content
        .find(|content| content.starts_with("{\"")) // JSON API data
        .and_then(|content| {
            println!("{}", content);
            serde_json::from_str::<GeniusAlbumResponse>(content).ok()
        })
        .ok_or("Failed to parse album info")?
        .album)
}
