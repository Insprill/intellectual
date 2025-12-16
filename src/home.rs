use actix_web::{HttpRequest, Responder, get};
use askama::Template;

use crate::{settings::Settings, settings::settings_from_req, templates::template};

#[derive(Template)]
#[template(path = "home.html")]
struct HomeTemplate {
    settings: Settings,
}

#[get("/")]
pub async fn home(req: HttpRequest) -> impl Responder {
    template(HomeTemplate {
        settings: settings_from_req(&req),
    })
}
