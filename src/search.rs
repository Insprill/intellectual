use std::cmp::{max, min};
use std::ops::RangeInclusive;

use actix_web::{get, web, Responder};
use askama::Template;
use serde::Deserialize;

use crate::genius;
use crate::genius::{GeniusSearchRequest, GeniusSong};
use crate::templates::template;
use crate::utils;

const NAV_PAGE_COUNT: i8 = 3;

#[derive(Template)]
#[template(path = "search.html")]
struct SearchTemplate {
    results: Vec<GeniusSong>,
    q: String,
    current_page: i8,
    nav_pages: Vec<i8>,
}

#[derive(Debug, Deserialize)]
pub struct SearchQuery {
    q: String,
    page: Option<i8>,
}

#[get("/search")]
pub async fn search(info: web::Query<SearchQuery>) -> impl Responder {
    let response = genius::text(genius::SubDomain::Api, &format!("search?q={}", info.q)).await;
    let deserialized: GeniusSearchRequest = serde_json::from_str(&response).unwrap();

    let current_page = info.page.unwrap_or_else(|| 1);
    let nav_min = max(1, current_page - NAV_PAGE_COUNT);
    let nav_max = min(100, current_page + NAV_PAGE_COUNT);
    let nav_pages = RangeInclusive::new(nav_min, nav_max).collect();

    template(SearchTemplate {
        q: info.q.to_string(),
        current_page,
        nav_pages,
        results: deserialized
            .response
            .hits
            .into_iter()
            .map(|x| x.result)
            .collect(),
    })
}
