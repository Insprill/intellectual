use actix_web::{get, HttpRequest, Responder};
use askama::Template;

use crate::{settings::settings_from_req, settings::Settings, templates::template};

#[derive(Template)]
#[template(path = "home.html")]
struct HomeTemplate {
    settings: Settings,
}

#[get("/")]
pub async fn home(req: HttpRequest) -> impl Responder {
    template(
        &req,
        HomeTemplate {
            settings: settings_from_req(&req),
        },
    )
}
