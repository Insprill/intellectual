use actix_web::{http::StatusCode, web::Bytes};
use awc::Client;
use serde::Deserialize;
use urlencoding::encode;

// region API

pub async fn text(subdomain: SubDomain, path: &str, queries: Option<Vec<(&str, &str)>>) -> String {
    String::from_utf8(request(subdomain, path, queries).await.1.to_vec()).unwrap()
}

pub async fn request(
    subdomain: SubDomain,
    path: &str,
    queries: Option<Vec<(&str, &str)>>,
) -> (StatusCode, Bytes) {
    let query_str = if let Some(q) = queries {
        String::from_iter(
            q.iter()
                .map(|query| format!("&{}={}", query.0, encode(query.1).into_owned())),
        )
    } else {
        "".into()
    };
    let mut client = Client::default().get(format!(
        "https://{}genius.com/{}?text_format=plain{}",
        subdomain.value(),
        path,
        query_str
    ));
    if matches!(subdomain, SubDomain::Api) {
        client = client.bearer_auth(std::env::var("GENIUS_AUTH_TOKEN").unwrap());
    }
    let mut res = client.send().await.unwrap();
    (res.status(), res.body().await.unwrap())
}

pub async fn post(subdomain: SubDomain, path: &str) -> StatusCode {
    Client::default()
        .get(format!("https://{}genius.com/{}", subdomain.value(), path))
        .bearer_auth(std::env::var("GENIUS_AUTH_TOKEN").unwrap())
        .send()
        .await
        .unwrap()
        .status()
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
pub struct GeniusAlbum {
    pub name: String,
}

#[derive(Deserialize)]
pub struct GeniusStats {
    pub pageviews: Option<i32>,
}

#[derive(Deserialize)]
pub struct GeniusArtist {
    pub api_path: String,
    pub name: String,
    pub alternate_names: Option<Vec<String>>,
    pub image_url: String,
    pub url: String,
    pub description: Option<GeniusDescription>,
    pub facebook_name: Option<String>,
    pub instagram_name: Option<String>,
    pub twitter_name: Option<String>,
}

impl GeniusArtist {
    pub fn socials(&self) -> Vec<ArtistSocial> {
        let mut socials = Vec::with_capacity(3);

        if let Some(name) = self.facebook_name.as_ref() {
            socials.push(ArtistSocial {
                name_raw: name,
                name_formatted: name.to_string(),
                brand: "facebook",
            })
        }

        if let Some(name) = self.instagram_name.as_ref() {
            socials.push(ArtistSocial {
                name_raw: name,
                name_formatted: format!("@{}", name),
                brand: "instagram",
            })
        }

        if let Some(name) = self.twitter_name.as_ref() {
            socials.push(ArtistSocial {
                name_raw: name,
                name_formatted: format!("@{}", name),
                brand: "twitter",
            })
        }

        socials
    }
}

#[derive(Deserialize)]
pub struct GeniusDescription {
    pub plain: String,
}

// endregion

pub struct ArtistSocial<'a> {
    pub name_raw: &'a str,
    pub name_formatted: String,
    pub brand: &'a str,
}
