use actix_web::{get, Responder, web};
use askama::Template;
use serde::Deserialize;

use crate::genius;
use crate::templates::template;

#[derive(Template)]
#[template(path = "search.html")]
struct SearchTemplate {
    results: Vec<GeniusResult>,
}

#[derive(Debug, Deserialize)]
pub struct SearchQuery {
    q: String,
}

#[get("/search")]
pub async fn search(info: web::Query<SearchQuery>) -> impl Responder {
    let response = genius::text(genius::SubDomain::Api, &format!("search?q={}", info.q)).await;
    let deserialized: GeniusSearch = serde_json::from_str(&response).unwrap();

    template(SearchTemplate {
        results: deserialized.response.hits.into_iter().map(|x| x.result).collect(),
    })
}

// region Genius Response
#[derive(Deserialize)]
struct GeniusSearch {
    response: GeniusResponse,
}

#[derive(Deserialize)]
struct GeniusResponse {
    hits: Vec<GeniusHit>,
}

#[derive(Deserialize)]
struct GeniusHit {
    result: GeniusResult,
}

#[allow(dead_code)]
#[derive(Deserialize)]
struct GeniusResult {
    title: String,
    artist_names: String,
    path: String,
    api_path: String,
    song_art_image_thumbnail_url: String,
    stats: GeniusStats,
}

#[allow(dead_code)]
#[derive(Deserialize)]
struct GeniusStats {
    pageviews: Option<i32>,
}
// endregion

pub fn pretty_format_num(num: i32) -> String {
    if num >= 1_000_000 {
        format!("{:.1}M", num as f32 / 1_000_000.0)
    } else if num >= 1_000 {
        format!("{}K", num / 1_000)
    } else {
        format!("{}", num)
    }
}
