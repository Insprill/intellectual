use actix_web::{get, HttpResponse, Responder};

#[get("/search")]
pub async fn search() -> impl Responder {
    HttpResponse::Ok().body("search")
}