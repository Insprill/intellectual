use actix_web::{
    dev::{self, ServiceResponse},
    middleware::ErrorHandlerResponse,
    HttpResponse, Result,
};
use askama::Template;

use crate::templates::template;

pub fn render_500<B>(res: dev::ServiceResponse<B>) -> Result<ErrorHandlerResponse<B>> {
    let err = res.response().error().unwrap().to_string();

    let new_response = template(InternalErrorTemplate { err });

    create(res, new_response)
}

pub fn render_404<B>(res: dev::ServiceResponse<B>) -> Result<ErrorHandlerResponse<B>> {
    let new_response = template(NotFoundTemplate {});

    create(res, new_response)
}

pub fn render_400<B>(res: dev::ServiceResponse<B>) -> Result<ErrorHandlerResponse<B>> {
    let err = res.response().error().unwrap().to_string();

    let new_response = template(BadRequestTemplate { err });

    create(res, new_response)
}

fn create<B>(
    res: dev::ServiceResponse<B>,
    new_response: HttpResponse,
) -> Result<ErrorHandlerResponse<B>> {
    Ok(ErrorHandlerResponse::Response(
        ServiceResponse::new(res.into_parts().0, new_response).map_into_right_body(),
    ))
}

#[derive(Template)]
#[template(path = "500.html")]
struct InternalErrorTemplate {
    err: String,
}

#[derive(Template)]
#[template(path = "404.html")]
struct NotFoundTemplate {}

#[derive(Template)]
#[template(path = "400.html")]
struct BadRequestTemplate {
    err: String,
}
