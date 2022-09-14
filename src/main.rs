use actix_web::{App, HttpServer};
use clap::{Arg, Command};

mod home;
mod lyrics;
mod search;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let matches = Command::new("intellectual")
        .version(env!("CARGO_PKG_VERSION"))
        .about("Alternate front-end for Genius written in Rust")
        .arg(Arg::new("address")
                 .short('a')
                 .long("address")
                 .value_name("ADDRESS")
                 .help("Sets the address to listen on")
                 .default_value("0.0.0.0")
                 .takes_value(true),
        )
        .arg(Arg::new("port")
                 .short('p')
                 .long("port")
                 .value_name("PORT")
                 .help("Sets the port to listen on")
                 .default_value("8080")
                 .takes_value(true),
        )
        .get_matches();

    let address = matches.value_of("address").unwrap_or("0.0.0.0"); //TODO: Validate this
    let port = std::env::var("PORT").unwrap_or_else(|_| matches.value_of("port").unwrap_or("8080").to_string()).parse::<u16>().unwrap(); //TODO: Validate this

    println!("Running Intellectual v{}, listening on {}:{}!", env!("CARGO_PKG_VERSION"), address, port);

    HttpServer::new(|| {
        App::new()
            .service(home::home)
            .service(search::search)
            .service(lyrics::lyrics)
    })
        .bind((address, port))?
        .run()
        .await
}
