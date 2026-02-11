use std::{sync::LazyLock, time::Duration};

use crate::Result;
use actix_web::{
    dev::{Decompress, Payload},
    http::{StatusCode, header::HeaderMap},
    web::Bytes,
};
use awc::{Client, ClientResponse};
use lazy_regex::*;
use log::debug;
use regex::Regex;
use scraper::{Html, Selector};
use serde::{Deserialize, Deserializer, de::DeserializeOwned};
use urlencoding::encode;

static EMBEDDED_INFO_SELECTOR: LazyLock<Selector> =
    LazyLock::new(|| Selector::parse("meta[content]").unwrap());

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

pub async fn get_annotation(id: i32) -> Result<GeniusReferentResponse> {
    Ok(get_json::<GeniusReferentRequest>(
        SubDomain::Api,
        &format!("referents/{id}"),
        Some(vec![("text_format", "html")]),
    )
    .await?
    .response)
}

// AWC default limit is 2MB.
// For some ungodly reason, some annotation descriptions have 13MB+ GIFs in them :|
const BODY_LIMIT: usize = 16 * 1024 * 1024;

pub async fn get_raw(
    subdomain: SubDomain,
    path: &str,
    queries: Option<Vec<(&str, &str)>>,
) -> Result<(StatusCode, Bytes, HeaderMap)> {
    let mut res = build_req(subdomain, path, queries).await?;
    Ok((
        res.status(),
        res.body().limit(BODY_LIMIT).await?,
        res.headers().clone(),
    ))
}

pub async fn get_text(
    subdomain: SubDomain,
    path: &str,
    queries: Option<Vec<(&str, &str)>>,
) -> Result<String> {
    let bytes = build_req(subdomain, path, queries)
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
    let mut res = build_req(subdomain, path, queries).await?;
    // We have to do this shit instead of just parsing as JSON since Genius,
    // at the time of writing, and as their name sarcasticly implies,
    // gives us a Content-Type of `application/html` for a JSON response!
    let body = &res.body().await?;
    let json_str = String::from_utf8_lossy(body);
    Ok(serde_json::from_str(&json_str)?)
}

// Default AWC timeout is 5 seconds (as of 9c70a88) which causes frequent timeouts.
const TIMEOUT_SECS: u64 = 30;

async fn build_req(
    subdomain: SubDomain,
    path: &str,
    queries: Option<Vec<(&str, &str)>>,
) -> Result<ClientResponse<Decompress<Payload>>> {
    let query_str = if let Some(q) = queries {
        String::from_iter(
            q.iter()
                .map(|query| format!("&{}={}", query.0, encode(query.1).into_owned())),
        )
    } else {
        "".into()
    };

    // Using the API path lets us drop the requirement for an API key.
    let path: String = if matches!(subdomain, SubDomain::Api) {
        format!("api/{path}")
    } else {
        path.to_owned()
    };

    let url = format!(
        "https://{}genius.com/{}?text_format=plain{}",
        subdomain.value(),
        path.trim_start_matches('/'),
        query_str
    );
    debug!("Sending request to {url}");

    let res = Client::builder()
        .timeout(Duration::from_secs(TIMEOUT_SECS))
        .add_default_header(("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/143.0.0.0 Safari/537.36"))
        .finish()
        .get(url)
        .send()
        .await?;
    let status = res.status();

    if status.is_client_error() || status.is_server_error() {
        Err(format!("Got response {status}").into())
    } else {
        Ok(res)
    }
}

static GENIUS_IMAGE_URL: &str = "https://images.genius.com/";
static GENIUS_IMAGE_ALT_URL: &str = "https://images.rapgenius.com/";
static GENIUS_BASE_PATTERN: Lazy<Regex> = lazy_regex!(r#"https?://\w*.?genius\.com"#);
static YOUTUBE_URL: &str = "youtube.com/";
static YOUTUBE_NOCOOKIE_URL: &str = "youtube-nocookie.com/";

pub fn rewrite_links<'de, D>(deserializer: D) -> std::result::Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let html = String::deserialize(deserializer)?;
    let html = html.replace(
        GENIUS_IMAGE_URL,
        &format!("/api/image?url={GENIUS_IMAGE_URL}"),
    ); // Images
    let html = html.replace(
        GENIUS_IMAGE_ALT_URL,
        &format!("/api/image?url={GENIUS_IMAGE_URL}"),
    ); // Images
    let html = html.replace(YOUTUBE_URL, YOUTUBE_NOCOOKIE_URL); // YouTube no cookie
    let html = GENIUS_BASE_PATTERN.replace_all(&html, ""); // We follow Genius' schema
    Ok(html.to_string())
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

#[derive(Deserialize, Debug)]
pub struct GeniusSearchRequest {
    pub response: GeniusSearchResponse,
}

#[derive(Deserialize, Debug)]
pub struct GeniusSearchResponse {
    pub hits: Vec<GeniusHit>,
}

#[derive(Deserialize, Debug)]
pub struct GeniusReferentRequest {
    pub response: GeniusReferentResponse,
}

#[derive(Deserialize, Debug)]
pub struct GeniusReferentResponse {
    pub referent: GeniusReferent,
}

#[derive(Deserialize, Debug)]
pub struct GeniusAnnotation {
    pub id: i32,
    pub body: GeniusAnnotationBody,
    pub votes_total: i32,
    // pub verified: bool, // TODO: indicate this in the UI
}

#[derive(Deserialize, Debug)]
pub struct GeniusAnnotationBody {
    #[serde(deserialize_with = "rewrite_links")]
    pub html: String,
}

#[derive(Deserialize, Debug)]
pub struct GeniusReferent {
    pub id: i32,
    pub fragment: String,
    pub annotations: Vec<GeniusAnnotation>,
}

#[derive(Deserialize, Debug)]
pub struct GeniusHit {
    pub result: GeniusSong,
}

#[derive(Deserialize, Debug)]
pub struct GeniusSongRequest {
    pub response: GeniusSongResponse,
}

#[derive(Deserialize, Debug)]
pub struct GeniusSongResponse {
    pub song: GeniusSong,
}

#[derive(Deserialize, Debug)]
pub struct GeniusSongsRequest {
    pub response: GeniusSongsResponse,
}

#[derive(Deserialize, Debug)]
pub struct GeniusSongsResponse {
    pub songs: Vec<GeniusSong>,
}

#[derive(Deserialize, Debug)]
pub struct GeniusSong {
    pub id: u32,
    pub title: String,
    pub path: String,
    pub header_image_url: String,
    pub release_date_for_display: Option<String>,
    pub song_art_image_thumbnail_url: String,
    pub album: Option<GeniusAlbum>,
    pub stats: GeniusStats,
    pub primary_artist: GeniusArtist,
}

#[derive(Deserialize, Debug)]
pub struct GeniusAlbumResponse {
    pub album: GeniusAlbum,
}

#[derive(Deserialize, Debug)]
pub struct GeniusTracksRequest {
    pub response: GeniusTracksResponse,
}

#[derive(Deserialize, Debug)]
pub struct GeniusTracksResponse {
    pub tracks: Vec<GeniusTrack>,
}

#[derive(Deserialize, Debug)]
pub struct GeniusTrack {
    pub song: GeniusSong,
}

#[derive(Deserialize, Debug)]
pub struct GeniusAlbum {
    pub name: String,
    pub id: u32,
    pub url: String,
    pub cover_art_url: String,
    pub release_date_for_display: Option<String>,
    pub tracks: Option<Vec<GeniusSong>>,
    pub artist: GeniusArtist,
}

#[derive(Deserialize, Debug)]
pub struct GeniusStats {
    pub pageviews: Option<i32>,
}

#[derive(Deserialize, Debug)]
pub struct GeniusArtistResponse {
    pub artist: GeniusArtist,
}

#[derive(Deserialize, Debug)]
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

#[derive(Deserialize, Debug)]
pub struct GeniusDescription {
    #[serde(deserialize_with = "rewrite_links")]
    pub html: String,
}

pub struct ArtistSocial<'a> {
    pub name_raw: &'a str,
    pub name_formatted: String,
    pub brand: &'static str,
}

impl GeniusArtist {
    pub fn socials(&self) -> Vec<ArtistSocial<'_>> {
        let mut socials = Vec::with_capacity(3);

        if let Some(name) = self.facebook_name.as_ref()
            && !name.is_empty()
        {
            socials.push(ArtistSocial {
                name_raw: name,
                name_formatted: name.to_string(),
                brand: "facebook",
            })
        }

        if let Some(name) = self.instagram_name.as_ref()
            && !name.is_empty()
        {
            socials.push(ArtistSocial {
                name_raw: name,
                name_formatted: format!("@{name}"),
                brand: "instagram",
            })
        }

        if let Some(name) = self.twitter_name.as_ref()
            && !name.is_empty()
        {
            socials.push(ArtistSocial {
                name_raw: name,
                name_formatted: format!("@{name}"),
                brand: "twitter",
            })
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
