use std::cmp::{max, min};
use std::ops::RangeInclusive;

use actix_web::{get, web, HttpRequest, Responder, Result};
use askama::Template;
use serde::Deserialize;

use crate::genius::{self, GeniusSong};
use crate::settings::{settings_from_req, Settings};
use crate::templates::template;
use crate::utils;

const NAV_PAGE_COUNT: u8 = 3;

#[derive(Template)]
#[template(path = "search.html")]
struct SearchTemplate {
    settings: Settings,
    songs: Vec<GeniusSong>,
    q: String,
    current_page: u8,
    nav_pages: Vec<u8>,
}

#[derive(Debug, Deserialize)]
pub struct SearchQuery {
    q: String,
    page: Option<u8>,
}

#[get("/search")]
pub async fn search(req: HttpRequest, info: web::Query<SearchQuery>) -> Result<impl Responder> {
    let current_page = info.page.unwrap_or(1);

    let songs = genius::get_search_results(&info.q, current_page).await?;

    let nav_min = max(1, current_page.saturating_sub(NAV_PAGE_COUNT));
    let nav_max = min(100, current_page.saturating_add(NAV_PAGE_COUNT));
    let nav_pages = RangeInclusive::new(nav_min, nav_max).collect();

    Ok(template(SearchTemplate {
        settings: settings_from_req(&req),
        q: info.q.to_owned(),
        current_page,
        nav_pages,
        songs,
    }))
}
