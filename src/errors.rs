use actix_web::{
    self, dev::ServiceResponse, middleware::ErrorHandlerResponse, HttpResponse, Result,
};
use askama::Template;

use crate::templates::template;

pub fn render_500<B>(res: ServiceResponse<B>) -> Result<ErrorHandlerResponse<B>> {
    let new_response = template(InternalErrorTemplate {
        err: get_err_str(&res),
    });

    create(res, new_response)
}

pub fn render_404<B>(res: ServiceResponse<B>) -> Result<ErrorHandlerResponse<B>> {
    let new_response = template(NotFoundTemplate {});

    create(res, new_response)
}

pub fn render_400<B>(res: ServiceResponse<B>) -> Result<ErrorHandlerResponse<B>> {
    let new_response = template(BadRequestTemplate {
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
    err: Option<String>,
}

#[derive(Template)]
#[template(path = "404.html")]
struct NotFoundTemplate {}

#[derive(Template)]
#[template(path = "400.html")]
struct BadRequestTemplate {
    err: Option<String>,
}
