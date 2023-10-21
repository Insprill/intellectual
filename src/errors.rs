use actix_web::{
    self, dev::ServiceResponse, middleware::ErrorHandlerResponse, HttpResponse, Result,
};
use askama::Template;
use log::error;

use crate::{templates::template, settings::{Settings, settings_from_req}};

pub fn render_500<B>(res: ServiceResponse<B>) -> Result<ErrorHandlerResponse<B>> {
    let err = get_err_str(&res);
    if let Some(str) = &err {
        error!("{}", str);
    }

    let new_response = template(InternalErrorTemplate { settings: settings_from_req(res.request()), err });
    create(res, new_response)
}

pub fn render_404<B>(res: ServiceResponse<B>) -> Result<ErrorHandlerResponse<B>> {
    let new_response = template(NotFoundTemplate { settings: settings_from_req(res.request())});
    create(res, new_response)
}

pub fn render_400<B>(res: ServiceResponse<B>) -> Result<ErrorHandlerResponse<B>> {
    let new_response = template(BadRequestTemplate {
        settings: settings_from_req(res.request()),
        err: get_err_str(&res),
    });
    create(res, new_response)
}

fn create<B>(
    res: ServiceResponse<B>,
    new_response: HttpResponse,
) -> Result<ErrorHandlerResponse<B>> {
    Ok(ErrorHandlerResponse::Response(
        ServiceResponse::new(res.into_parts().0, new_response).map_into_right_body(),
    ))
}

fn get_err_str<B>(res: &ServiceResponse<B>) -> Option<String> {
    res.response().error().map(|err| err.to_string())
}

#[derive(Template)]
#[template(path = "500.html")]
struct InternalErrorTemplate {
    settings: Settings, 
    err: Option<String>,
}

#[derive(Template)]
#[template(path = "404.html")]
struct NotFoundTemplate {
    settings: Settings
}

#[derive(Template)]
#[template(path = "400.html")]
struct BadRequestTemplate {
    settings: Settings,
    err: Option<String>,
}
