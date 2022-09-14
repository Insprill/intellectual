use std::io;

use actix_files::NamedFile;
use actix_web::{get};

#[get("/")]
pub async fn home() -> io::Result<NamedFile> {
    Ok(NamedFile::open("static/home.html")?)
}
