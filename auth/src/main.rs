#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_migrations;
#[macro_use]
extern crate diesel_derive_enum;

use log::info;
use tonic::transport::Server;
use pb::auth_server::AuthServer;

mod config;
mod confirms;
mod models;
mod proto_convert;
mod repo;
mod schema;
mod server;
mod service;

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();
    env_logger::init();

    let cfg = config::Settings::new().expect("Failed to parse config");
    let repo = repo::PgRepo::new(&cfg).expect("Failed to initialize repo");
    let confirms_sender = confirms::ConfrimsSender::new(&cfg).await.expect("Failed to initialize confirmations sender");
    let auth_service = service::Service::new(&cfg, repo, confirms_sender);
    let auth_client = auth_client::client::Client::new(&cfg.bind_address.to_string()).expect("Failed to initalize auth client");
    let server = server::Server::new(auth_service, auth_client);

    info!("Starting grpc server at {}", cfg.bind_address);
    Server::builder()
        .add_service(AuthServer::new(server))
        .serve(cfg.bind_address)
        .await?;

    Ok(())
}
