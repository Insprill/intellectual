use actix_web::web::Bytes;
use reqwest::{Client, Response};

pub async fn text(subdomain: SubDomain, path: &str) -> String {
    request(subdomain, path).await.text().await.unwrap()
}

pub async fn bytes(subdomain: SubDomain, path: &str) -> Bytes {
    request(subdomain, path).await.bytes().await.unwrap()
}

async fn request(subdomain: SubDomain, path: &str) -> Response {
    Client::new()
        .get(format!("https://{}genius.com/{}", subdomain.value(), path))
        .header("Authorization", format!("Bearer {}", std::env::var("GENIUS_AUTH_TOKEN").unwrap()))
        .send()
        .await.unwrap()
}

pub enum SubDomain {
    API,
    IMAGES,
    ROOT,
}

impl SubDomain {
    fn value(&self) -> &str {
        match *self {
            SubDomain::API => "api.",
            SubDomain::IMAGES => "images.",
            SubDomain::ROOT => "",
        }
    }
}
