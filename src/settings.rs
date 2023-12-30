use actix_web::{get, post, web::Form, HttpRequest, HttpResponse, Responder};
use askama::Template;
use cookie::Cookie;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};

use crate::templates::template;

static THEME_CONFIG: Lazy<ThemeConfig> =
    Lazy::new(|| serde_json::from_str(include_str!("../static/style/theme/themes.json")).unwrap());

#[derive(Template)]
#[template(path = "settings.html")]
struct SettingsTemplate {
    settings: Settings,
    themes: Vec<Theme>,
}

#[get("/settings")]
pub async fn settings(req: HttpRequest) -> impl Responder {
    template(SettingsTemplate {
        settings: settings_from_req(&req),
        themes: THEME_CONFIG.themes.clone(),
    })
}

#[post("/settings")]
pub async fn settings_form(form: Form<Settings>) -> impl Responder {
    match serde_json::to_string(&form) {
        Ok(str) => HttpResponse::SeeOther()
            .cookie(Cookie::new("settings", str))
            .append_header(("Location", "/settings"))
            .finish(),
        Err(_) => HttpResponse::BadRequest().finish(),
    }
}

#[derive(Serialize, Deserialize)]
pub struct Settings {
    pub theme: String,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            theme: "github-dark".into(),
        }
    }
}

impl Settings {
    pub fn is_valid(&self) -> bool {
        THEME_CONFIG.themes.iter().any(|t| t.id == self.theme)
    }
}

pub fn settings_from_req(req: &HttpRequest) -> Settings {
    req.cookie("settings")
        .and_then(|cookie| serde_json::from_str::<Settings>(cookie.value()).ok())
        .filter(|s| s.is_valid())
        .unwrap_or_default()
}

#[derive(Clone, Deserialize)]
struct Theme {
    id: String,
    name: String,
}

#[derive(Deserialize)]
struct ThemeConfig {
    themes: Vec<Theme>,
}
