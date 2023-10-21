use actix_web::{get, Responder, HttpRequest};
use askama::Template;

use crate::{templates::template, settings::Settings, settings::settings_from_req};

#[derive(Template)]
#[template(path = "home.html")]
struct HomeTemplate {
    settings: Settings
}

#[get("/")]
pub async fn home(req: HttpRequest) -> impl Responder {
    template(HomeTemplate {
        settings: settings_from_req(&req)
    })
}
