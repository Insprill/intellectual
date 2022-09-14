use actix_web::{get, HttpResponse, Responder};

#[get("/lyrics")]
pub async fn lyrics() -> impl Responder {
    HttpResponse::Ok().body("lyrics")
}