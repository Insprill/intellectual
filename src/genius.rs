use actix_web::web::Bytes;
use reqwest::{Client, Response};
use serde::Deserialize;

// region API

pub async fn text(subdomain: SubDomain, path: &str) -> String {
    request(subdomain, path).await.text().await.unwrap()
}

pub async fn bytes(subdomain: SubDomain, path: &str) -> Bytes {
    request(subdomain, path).await.bytes().await.unwrap()
}

async fn request(subdomain: SubDomain, path: &str) -> Response {
    let mut builder = Client::new()
        .get(format!("https://{}genius.com/{}", subdomain.value(), path))
        .header("Authorization", format!("Bearer {}", std::env::var("GENIUS_AUTH_TOKEN").unwrap()));
    if matches!(subdomain, SubDomain::Api) {
        builder = builder.query(&[("text_format", "plain")]);
    }
    builder.send().await.unwrap()
}

pub enum SubDomain {
    Api,
    Images,
    Root,
}

impl SubDomain {
    fn value(&self) -> &str {
        match *self {
            SubDomain::Api => "api.",
            SubDomain::Images => "images.",
            SubDomain::Root => "",
        }
    }
}

// endregion

// region Structs

// region Search Endpoint
#[derive(Deserialize)]
pub struct GeniusSearchRequest {
    pub response: GeniusSearchResponse,
}

#[derive(Deserialize)]
pub struct GeniusSearchResponse {
    pub hits: Vec<GeniusHit>,
}

#[derive(Deserialize)]
pub struct GeniusHit {
    pub result: GeniusSong,
}
// endregion

// region Song Endpoint
#[derive(Deserialize)]
pub struct GeniusSongRequest {
    pub response: GeniusSongResponse,
}

#[derive(Deserialize)]
pub struct GeniusSongResponse {
    pub song: GeniusSong,
}
// endregion

// region Artist Endpoint
#[derive(Deserialize)]
pub struct GeniusArtistRequest {
    pub response: GeniusArtistResponse,
}

#[derive(Deserialize)]
pub struct GeniusArtistResponse {
    pub artist: GeniusArtist,
}
// endregion

#[derive(Deserialize)]
pub struct GeniusSong {
    pub id: u32,
    pub title: String,
    pub artist_names: String,
    pub path: String,
    pub header_image_url: String,
    pub release_date_for_display: Option<String>,
    pub song_art_image_thumbnail_url: String,
    pub api_path: String,
    pub album: Option<GeniusAlbum>,
    pub stats: GeniusStats,
}

#[derive(Deserialize)]
pub struct GeniusAlbum {
    pub name: String,
}

#[derive(Deserialize)]
pub struct GeniusStats {
    pub pageviews: Option<i32>,
}

#[derive(Deserialize)]
pub struct GeniusArtist {
    pub name: String,
    pub alternate_names: Option<Vec<String>>,
    pub image_url: String,
    pub url: String,
}

// endregion
