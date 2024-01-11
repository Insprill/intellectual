#![forbid(unsafe_code)]

use std::{error::Error, fs::File, io::BufReader, process::exit, time::Duration};

use actix_web::{http::StatusCode, middleware, App, HttpServer};
use clap::{arg, command, Parser};
use log::{error, info, warn, LevelFilter};
use rustls::{Certificate, PrivateKey, ServerConfig as RustlsServerConfig};

use simplelog::{ColorChoice, CombinedLogger, Config, TermLogger, TerminalMode};

mod album;
mod api;
mod artist;
mod errors;
mod genius;
mod home;
mod lyrics;
mod resource;
mod search;
mod settings;
mod templates;
mod utils;

pub type Result<T> = std::result::Result<T, Box<dyn Error>>;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Sets the address to listen on
    #[arg(short, long, default_value = "0.0.0.0")]
    address: String,

    /// Sets the port to listen on. Will be overriden by the PORT env var if present
    #[arg(short, long, default_value_t = 8080)]
    port: u16,

    /// The amount of HTTP workers to use. 0 to equal physical CPU cores
    #[arg(short, long, default_value_t = 0)]
    workers: usize,

    /// The Keep-Alive timeout, in seconds. Set to 0 to disable.
    #[arg(short, long, default_value_t = 15.0)]
    keep_alive_timeout: f32,

    /// Whether TLS should be used
    #[arg(long, default_value = "false")]
    tls: bool,

    /// The path to the KEY file. Required when using TLS.
    #[arg(long, required_if_eq("tls", "true"))]
    tls_key_file: Option<String>,

    /// The path to the CERT file. Required when using TLS.
    #[arg(long, required_if_eq("tls", "true"))]
    tls_cert_file: Option<String>,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    CombinedLogger::init(vec![TermLogger::new(
        LevelFilter::Info,
        Config::default(),
        TerminalMode::Mixed,
        ColorChoice::Auto,
    )])
    .unwrap();

    let args = Args::parse();

    let port = std::env::var("PORT")
        .unwrap_or_else(|_| args.port.to_string())
        .parse::<u16>()
        .unwrap();

    info!(
        "Starting Intellectual v{}, listening on {}:{}!",
        env!("IN_VERSION"),
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
            .service(album::album)
            .service(api::image)
            .service(artist::artist)
            .service(home::home)
            .service(lyrics::lyrics)
            .service(search::search)
            .service(settings::settings)
            .service(settings::settings_form)
            // Static Resources
            .service(resource::resource)
            .service(resource::style)
            .service(resource::style_theme)
            .service(resource::icon)
            .service(resource::font)
    });

    if args.keep_alive_timeout > 0.0 {
        server = server.keep_alive(Duration::from_secs_f32(args.keep_alive_timeout));
    } else {
        server = server.keep_alive(None)
    }

    if args.workers > 0 {
        server = server.workers(args.workers);
    }

    if args.tls {
        // To create a self-signed temporary cert for testing:
        // openssl req -x509 -newkey rsa:4096 -nodes -keyout key.pem -out cert.pem -days 365 -subj '/CN=localhost'
        server.bind_rustls_021((args.address.to_owned(), port), build_tls_config(&args)?)
    } else {
        server.bind_auto_h2c((args.address, port))
    }?
    .run()
    .await
}

fn build_tls_config(args: &Args) -> std::io::Result<RustlsServerConfig> {
    Ok(RustlsServerConfig::builder()
        .with_safe_defaults()
        .with_no_client_auth()
        .with_single_cert(create_cert_chain(args), PrivateKey(create_key(args)))
        .unwrap())
}

fn create_cert_chain(args: &Args) -> Vec<Certificate> {
    let cert_file_path = args.tls_cert_file.as_ref().unwrap();
    let cert_file = &mut BufReader::new(match File::open(cert_file_path) {
        Ok(file) => file,
        Err(err) => {
            error!("Failed to load cert file '{}': {}", cert_file_path, err);
            exit(1);
        }
    });

    let cert_chain: Vec<Certificate> = rustls_pemfile::certs(cert_file)
        .unwrap()
        .into_iter()
        .map(Certificate)
        .collect();
    if cert_chain.is_empty() {
        error!("Failed to find any certs in '{}'", cert_file_path);
        exit(1);
    }
    cert_chain
}

fn create_key(args: &Args) -> Vec<u8> {
    let key_file_path = args.tls_key_file.as_ref().unwrap();
    let key_file = &mut BufReader::new(match File::open(key_file_path) {
        Ok(file) => file,
        Err(err) => {
            error!("Failed to load key file '{}': {}", key_file_path, err);
            exit(1);
        }
    });
    let mut keys: Vec<Vec<u8>> = rustls_pemfile::pkcs8_private_keys(key_file).unwrap();
    if keys.is_empty() {
        error!("Failed to find any keys in '{}'", key_file_path);
        exit(1);
    }
    if keys.len() > 1 {
        warn!(
            "Found multiple keys in '{}'! Only the first will be used.",
            key_file_path
        );
    }

    keys.remove(0)
}
