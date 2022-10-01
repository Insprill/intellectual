use actix_web::{get, Responder, web};
use askama::Template;
use serde::Deserialize;

use crate::genius;
use crate::genius::{GeniusSearchRequest, GeniusSong};
use crate::templates::template;
use crate::utils;

#[derive(Template)]
#[template(path = "search.html")]
struct SearchTemplate {
    results: Vec<GeniusSong>,
}

#[derive(Debug, Deserialize)]
pub struct SearchQuery {
    q: String,
}

#[get("/search")]
pub async fn search(info: web::Query<SearchQuery>) -> impl Responder {
    let response = genius::text(genius::SubDomain::Api, &format!("search?q={}", info.q)).await;
    let deserialized: GeniusSearchRequest = serde_json::from_str(&response).unwrap();

    template(SearchTemplate {
        results: deserialized.response.hits.into_iter().map(|x| x.result).collect(),
    })
}
