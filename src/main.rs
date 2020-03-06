#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_migrations;

use actix_web::{delete, get, middleware, post, put, web, App, Error, HttpResponse, HttpServer};
use log::info;

mod models;
mod schema;
mod store;

#[post("/v1/product")]
async fn add_product(
    store: web::Data<store::Store>,
    data: web::Json<models::NewProduct>,
) -> Result<HttpResponse, Error> {
    let product = web::block(move || store.add_product(data.0)).await?;

    Ok(HttpResponse::Created().json(product))
}

#[get("/v1/product/{id}")]
async fn get_product(
    store: web::Data<store::Store>,
    id: web::Path<i32>,
) -> Result<HttpResponse, Error> {
    let product_id = *id;

    let product = web::block(move || store.get_product(product_id)).await?;

    match product {
        Some(result) => Ok(HttpResponse::Ok().json(result)),
        None => Ok(HttpResponse::NotFound().json(models::ApiError {
            error: format!("Cannot get product {}", product_id),
        })),
    }
}

#[put("/v1/product/{id}")]
async fn update_product(
    store: web::Data<store::Store>,
    id: web::Path<i32>,
    data: web::Json<models::NewProduct>,
) -> Result<HttpResponse, Error> {
    let product = web::block(move || store.update_product(*id, data.0)).await?;

    Ok(HttpResponse::Ok().json(product))
}

#[delete("/v1/product/{id}")]
async fn remove_product(
    store: web::Data<store::Store>,
    id: web::Path<i32>,
) -> Result<HttpResponse, Error> {
    let product_id = *id;
    let num_removed = web::block(move || store.remove_product(product_id)).await?;
    match num_removed {
        0 => Ok(HttpResponse::NotFound().json(models::ApiError {
            error: format!("Cannot find product with id {}", product_id),
        })),
        n => Ok(HttpResponse::Ok().json(models::ApiSuccess {
            status: format!("Removed {} products", n),
        })),
    }
}

#[get("/v1/products")]
async fn list_products(
    store: web::Data<store::Store>,
    web::Query(query): web::Query<models::ListQuery>,
) -> Result<HttpResponse, Error> {
    let products = web::block(move || store.list_products(query)).await?;
    Ok(HttpResponse::Ok().json(products))
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();

    let address = std::env::var("BIND_ADDRESS").unwrap_or_else(|_| String::from("0.0.0.0:8080"));
    let store = store::Store::new();

    info!("Starting server at {}", &address);
    HttpServer::new(move || {
        App::new()
            .data(store.clone())
            .wrap(middleware::Logger::default())
            .service(add_product)
            .service(get_product)
            .service(update_product)
            .service(remove_product)
            .service(list_products)
    })
    .bind(address)?
    .run()
    .await
}
