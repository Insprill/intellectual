use actix_web::{get, HttpResponse, Responder};

#[get("/")]
pub async fn home() -> impl Responder {
    HttpResponse::Ok().body("home")
}