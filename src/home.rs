use actix_web::{get, Responder};
use askama::Template;

use crate::templates::template;

#[derive(Template)]
#[template(path = "home.html")]
struct HomeTemplate {}

#[get("/")]
pub async fn home() -> impl Responder {
    template(HomeTemplate {})
}
