#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_migrations;

use log::info;
use tonic::transport::Server;
use pb::shop_server::ShopServer;

mod config;
mod importer;
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
    let auth = auth_client::client::Client::new(&cfg.auth_address)?;
    let repo = repo::PgRepo::new(&cfg).expect("Failed to initialize repo");
    let service = service::Service::new(repo.clone(), auth);
    let server = server::Server::new(service);
    let srv = Server::builder().add_service(ShopServer::new(server));

    let bind_address = cfg.bind_address;
    let importer = tokio::task::spawn_blocking(|| {
        let mut importer = importer::Importer::new(repo, cfg);
        futures::executor::block_on(importer.run()).unwrap();
    });

    info!("Starting grpc server at {}", bind_address);

    srv.serve(bind_address).await?;
    importer.await?;

    Ok(())
}
