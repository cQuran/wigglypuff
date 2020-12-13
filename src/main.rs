use env_logger;
use clap::{crate_authors, crate_version, Arg};
use std::{env, io};

use actix_web::{App, HttpServer};

mod api;
mod config;
mod constants;
mod models;

#[actix_web::main]
async fn main() -> io::Result<()> {
    let input_arguments = clap::App::new("Wigglypuff")
        .version(crate_version!())
        .author(crate_authors!())
        .about("Service for reading wigglypuff sura and ayat")
        .args(&[
            Arg::with_name("host")
                .help("Set wigglypuff-service host address")
                .env("HOST"),
            Arg::with_name("port")
                .help("Set wigglypuff-service port")
                .env("PORT"),
        ])
        .get_matches();

    env::set_var("RUST_LOG", "info,actix_web=debug");
    env_logger::init();
    let host = input_arguments.value_of("host").unwrap();
    let port = input_arguments.value_of("port").unwrap();
    let url = format!("{}:{}", &host, &port);


    HttpServer::new(move || {
        App::new()
            .wrap(actix_web::middleware::Logger::default())
            .configure(config::app::config_services)
    })
    .bind(&url)?
    .run()
    .await
}
