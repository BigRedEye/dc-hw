#[macro_use]
extern crate diesel;

use actix_web::{get, post, put, delete, web, middleware, App, Error, HttpServer, HttpResponse};
use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};
use log::{info, error};

mod models;
mod schema;
mod store;

type DbPool = r2d2::Pool<ConnectionManager<PgConnection>>;

fn handle_server_error<E>(e: E) -> HttpResponse
    where E: std::fmt::Display
{
    HttpResponse::InternalServerError().json( models::ApiError {
        error: format!("Internal server error: {}", e)
    })
}

#[post("/v1/product")]
async fn add_product(
    pool: web::Data<DbPool>,
    data: web::Json<models::NewProduct>
) -> Result<HttpResponse, Error> {
    let connection = pool.get().expect("Failed to get db connection");

    let product = web::block(move || store::add_product(data.0, &connection)).await.map_err(|e| {
        error!("Failed to get product: {}", e);
        handle_server_error(e)
    })?;

    Ok(HttpResponse::Created().json(product))
}

#[get("/v1/product/{id}")]
async fn get_product(
    pool: web::Data<DbPool>,
    id: web::Path<i32>,
) -> Result<HttpResponse, Error> {
    let connection = pool.get().expect("Failed to get db connection");

    let product_id = *id;

    let product = web::block(move || store::get_product(product_id.clone(), &connection)).await.map_err(|e| {
        error!("Failed to get product: {}", e);
        handle_server_error(e)
    })?;

    match product {
        Some(result) => Ok(HttpResponse::Ok().json(result)),
        None => Ok(HttpResponse::NotFound().json(models::ApiError {
            error: format!("Cannot get product {}", product_id)
        })),
    }
}

#[put("/v1/product/{id}")]
async fn update_product(
    pool: web::Data<DbPool>,
    id: web::Path<i32>,
    data: web::Json<models::NewProduct>
) -> Result<HttpResponse, Error> {
    let connection = pool.get().expect("Failed to get db connection");

    let product = web::block(move || store::update_product(*id, data.0, &connection)).await.map_err(|e| {
        error!("Failed to get product: {}", e);
        handle_server_error(e)
    })?;

    Ok(HttpResponse::Ok().json(product))
}

#[delete("/v1/product/{id}")]
async fn remove_product(
    pool: web::Data<DbPool>,
    id: web::Path<i32>,
) -> Result<HttpResponse, Error> {
    let connection = pool.get().expect("Failed to get db connection");
    let product_id = *id;

    let num_removed = web::block(move || store::remove_product(product_id, &connection)).await.map_err(|e| {
        error!("Failed to remove product: {}", e);
        handle_server_error(e)
    })?;

    match num_removed {
        0 => Ok(HttpResponse::NotFound().json(models::ApiError {
            error: format!("Cannot find product with id {}", product_id)
        })),
        _ => Ok(HttpResponse::Ok().finish())
    }
}

#[get("/v1/products")]
async fn list_products(
    pool: web::Data<DbPool>,
    web::Query(query): web::Query<models::ListQuery>,
) -> Result<HttpResponse, Error> {
    let connection = pool.get().expect("Failed to get db connection");

    let products = web::block(move || store::list_products(query.limit, query.offset, &connection)).await.map_err(|e| {
        error!("Failed to list products: {}", e);
        handle_server_error(e)
    })?;

    Ok(HttpResponse::Ok().json(products))
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "actix_web=info,diesel=debug");
    env_logger::init();
    dotenv::dotenv().ok();

    let connspec = std::env::var("DATABASE_URL").expect("DATABASE_URL");
    let manager = ConnectionManager::<PgConnection>::new(connspec);
    let pool = r2d2::Pool::builder()
        .build(manager)
        .expect("Failed to create pool");

    let address = "0.0.0.0:8080";
    info!("Starting server at {}", &address);

    HttpServer::new(move || {
        App::new()
            .data(pool.clone())
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
