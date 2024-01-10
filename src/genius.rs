use crate::Result;
use actix_web::{http::{StatusCode, header::HeaderMap}, web::Bytes};
use awc::{Client, SendClientRequest};
use once_cell::sync::Lazy;
use scraper::{Html, Selector};
use serde::{de::DeserializeOwned, Deserialize};
use urlencoding::encode;

static EMBEDDED_INFO_SELECTOR: Lazy<Selector> =
    Lazy::new(|| Selector::parse("meta[content]").unwrap());

pub async fn extract_data<Res>(path: &str) -> Result<Res>
where
    Res: DeserializeOwned,
{
    let page = get_text(SubDomain::Root, path, None).await?;
    let document = Html::parse_document(&page);

    Ok(document
        .select(&EMBEDDED_INFO_SELECTOR)
        .map(|element| element.value().attr("content").unwrap()) // Selector only matches content
        .find(|content| content.starts_with("{\"")) // JSON API data
        .and_then(|content| serde_json::from_str::<Res>(content).ok())
        .ok_or("Failed to extract JSON data")?)
}

/// https://docs.genius.com/#/artists-songs
pub async fn get_artist_songs(
    artist_id: u32,
    sort_mode: SortMode,
    limit: u8,
) -> Result<Vec<GeniusSong>> {
    Ok(get_json::<GeniusSongsRequest>(
        SubDomain::Api,
        &format!("artists/{artist_id}/songs"),
        Some(vec![sort_mode.to_query(), ("per_page", &limit.to_string())]),
    )
    .await?
    .response
    .songs)
}

pub async fn get_album_tracks(album_id: u32) -> Result<Vec<GeniusSong>> {
    Ok(
        get_json::<GeniusTracksRequest>(SubDomain::Api, &format!("albums/{album_id}/tracks"), None)
            .await?
            .response
            .tracks
            .into_iter()
            .map(|track| track.song)
            .collect(),
    )
}

/// https://docs.genius.com/#/songs-show
pub async fn get_song(song_id: u32) -> Result<GeniusSong> {
    Ok(
        get_json::<GeniusSongRequest>(SubDomain::Api, &format!("songs/{song_id}"), None)
            .await?
            .response
            .song,
    )
}

/// https://docs.genius.com/#/search-search
pub async fn get_search_results(query: &str, page: u8) -> Result<Vec<GeniusSong>> {
    Ok(get_json::<GeniusSearchRequest>(
        SubDomain::Api,
        "search",
        Some(vec![("q", query), ("page", &page.to_string())]),
    )
    .await?
    .response
    .hits
    .into_iter()
    .map(|x| x.result)
    .collect())
}

pub async fn get_raw(
    subdomain: SubDomain,
    path: &str,
    queries: Option<Vec<(&str, &str)>>,
) -> Result<(StatusCode, Bytes, HeaderMap)> {
    let mut res = build_req(subdomain, path, queries)?.await?;
    Ok((res.status(), res.body().await?, res.headers().clone()))
}

pub async fn get_text(
    subdomain: SubDomain,
    path: &str,
    queries: Option<Vec<(&str, &str)>>,
) -> Result<String> {
    let bytes = build_req(subdomain, path, queries)?
        .await?
        .body()
        .await?
        .to_vec();
    Ok(String::from_utf8(bytes)?)
}

async fn get_json<T: DeserializeOwned>(
    subdomain: SubDomain,
    path: &str,
    queries: Option<Vec<(&str, &str)>>,
) -> Result<T> {
    Ok(build_req(subdomain, path, queries)?
        .await?
        .json::<T>()
        .await?)
}

fn build_req(
    subdomain: SubDomain,
    path: &str,
    queries: Option<Vec<(&str, &str)>>,
) -> Result<SendClientRequest> {
    let query_str = if let Some(q) = queries {
        String::from_iter(
            q.iter()
                .map(|query| format!("&{}={}", query.0, encode(query.1).into_owned())),
        )
    } else {
        "".into()
    };

    // Using the api path lets us drop the requirement for an API key.
    let path: String = if matches!(subdomain, SubDomain::Api) {
        format!("api/{path}")
    } else {
        path.to_owned()
    };

    let req = Client::default()
        .get(format!(
            "https://{}genius.com/{}?text_format=plain{}",
            subdomain.value(),
            path.trim_start_matches('/'),
            query_str
        ))
        .send();

    Ok(req)
}

pub enum SubDomain {
    Api,
    Images,
    Root,
}

impl SubDomain {
    fn value(&self) -> &'static str {
        match *self {
            SubDomain::Images => "images.",
            SubDomain::Root => "",
            SubDomain::Api => "",
        }
    }
}

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

#[derive(Deserialize)]
pub struct GeniusSongRequest {
    pub response: GeniusSongResponse,
}

#[derive(Deserialize)]
pub struct GeniusSongResponse {
    pub song: GeniusSong,
}

#[derive(Deserialize)]
pub struct GeniusSongsRequest {
    pub response: GeniusSongsResponse,
}

#[derive(Deserialize)]
pub struct GeniusSongsResponse {
    pub songs: Vec<GeniusSong>,
}

#[derive(Deserialize)]
pub struct GeniusSong {
    pub id: u32,
    pub title: String,
    pub path: String,
    pub header_image_url: String,
    pub release_date_for_display: Option<String>,
    pub song_art_image_thumbnail_url: String,
    pub api_path: String,
    pub album: Option<GeniusAlbum>,
    pub stats: GeniusStats,
    pub primary_artist: GeniusArtist,
}

#[derive(Deserialize)]
pub struct GeniusAlbumRequest {
    pub response: GeniusAlbumResponse,
}

#[derive(Deserialize)]
pub struct GeniusAlbumResponse {
    pub album: GeniusAlbum,
}

#[derive(Deserialize)]
pub struct GeniusTracksRequest {
    pub response: GeniusTracksResponse,
}

#[derive(Deserialize)]
pub struct GeniusTracksResponse {
    pub tracks: Vec<GeniusTrack>,
}

#[derive(Deserialize)]
pub struct GeniusTrack {
    pub song: GeniusSong,
}

#[derive(Deserialize)]
pub struct GeniusAlbum {
    pub name: String,
    pub id: u32,
    pub url: String,
    pub cover_art_url: String,
    pub release_date_for_display: Option<String>,
    pub tracks: Option<Vec<GeniusSong>>,
    pub artist: GeniusArtist,
}

#[derive(Deserialize)]
pub struct GeniusStats {
    pub pageviews: Option<i32>,
}

#[derive(Deserialize)]
pub struct GeniusArtistRequest {
    pub response: GeniusArtistResponse,
}

#[derive(Deserialize)]
pub struct GeniusArtistResponse {
    pub artist: GeniusArtist,
}

#[derive(Deserialize)]
pub struct GeniusArtist {
    pub id: u32,
    pub name: String,
    pub alternate_names: Option<Vec<String>>,
    pub image_url: String,
    pub url: String,
    pub description: Option<GeniusDescription>,
    pub popular_songs: Option<Vec<GeniusSong>>,
    pub facebook_name: Option<String>,
    pub instagram_name: Option<String>,
    pub twitter_name: Option<String>,
}

#[derive(Deserialize)]
pub struct GeniusDescription {
    pub html: String,
}

pub struct ArtistSocial<'a> {
    pub name_raw: &'a str,
    pub name_formatted: String,
    pub brand: &'static str,
}

impl GeniusArtist {
    pub fn socials(&self) -> Vec<ArtistSocial> {
        let mut socials = Vec::with_capacity(3);

        if let Some(name) = self.facebook_name.as_ref() {
            if !name.is_empty() {
                socials.push(ArtistSocial {
                    name_raw: name,
                    name_formatted: name.to_string(),
                    brand: "facebook",
                })
            }
        }

        if let Some(name) = self.instagram_name.as_ref() {
            if !name.is_empty() {
                socials.push(ArtistSocial {
                    name_raw: name,
                    name_formatted: format!("@{name}"),
                    brand: "instagram",
                })
            }
        }

        if let Some(name) = self.twitter_name.as_ref() {
            if !name.is_empty() {
                socials.push(ArtistSocial {
                    name_raw: name,
                    name_formatted: format!("@{name}"),
                    brand: "twitter",
                })
            }
        }

        socials
    }
}

pub enum SortMode {
    #[allow(dead_code)]
    Title,
    Popularity,
}

impl SortMode {
    pub fn to_query(&self) -> (&str, &str) {
        (
            "sort",
            match self {
                Self::Title => "title",
                Self::Popularity => "popularity",
            },
        )
    }
}
