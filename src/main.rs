use std::process::exit;

use actix_web::{http::StatusCode, middleware, App, HttpServer};
use clap::{arg, command, Parser};
use log::{error, info, LevelFilter};
use simplelog::{ColorChoice, CombinedLogger, Config, TermLogger, TerminalMode};

mod api;
mod artist;
mod errors;
mod genius;
mod home;
mod lyrics;
mod resource;
mod search;
mod templates;
mod utils;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Sets the address to listen on
    #[arg(short, long, default_value = "localhost")]
    address: String,

    /// Sets the port to listen on. Will be overriden by the PORT env var if present
    #[arg(short, long, default_value_t = 8080)]
    port: u16,

    /// The amount of HTTP workers to use. 0 to equal physical CPU cores
    #[arg(short, long, default_value_t = 0)]
    workers: usize,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let args = Args::parse();

    let port = std::env::var("PORT")
        .unwrap_or_else(|_| args.port.to_string())
        .parse::<u16>()
        .unwrap();

    CombinedLogger::init(vec![TermLogger::new(
        LevelFilter::Info,
        Config::default(),
        TerminalMode::Mixed,
        ColorChoice::Auto,
    )])
    .unwrap();

    if std::env::var("GENIUS_AUTH_TOKEN").is_err() {
        error!("GENIUS_AUTH_TOKEN environment variable not set!");
        exit(1);
    }

    info!(
        "Running Intellectual v{}, listening on {}:{}!",
        env!("CARGO_PKG_VERSION"),
        args.address,
        port
    );

    let mut server = HttpServer::new(|| {
        App::new()
            .wrap(
                middleware::ErrorHandlers::new()
                    .handler(StatusCode::INTERNAL_SERVER_ERROR, errors::render_500)
                    .handler(StatusCode::NOT_FOUND, errors::render_404)
                    .handler(StatusCode::BAD_REQUEST, errors::render_400),
            )
            .wrap(middleware::Compress::default())
            .wrap(
                middleware::DefaultHeaders::new()
                    .add(("Referrer-Policy", "no-referrer"))
                    .add(("X-Frame-Options", "DENY"))
                    .add(("X-Content-Type-Options", "nosniff"))
                    .add(("Content-Security-Policy", "default-src 'self'")),
            )
            // Routes
            .service(api::image)
            .service(artist::artist)
            .service(home::home)
            .service(lyrics::lyrics)
            .service(search::search)
            // Static Resources
            .service(resource::resource)
            .service(resource::style)
            .service(resource::icon)
            .service(resource::font)
    });

    if args.workers > 0 {
        server = server.workers(args.workers);
    }

    server.bind((args.address, port))?.run().await
}
