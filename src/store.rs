use actix_web::{error, http::StatusCode, HttpResponse};
use diesel::prelude::*;
use diesel::r2d2::ConnectionManager;
use log::info;
use thiserror::Error;

use crate::models;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Cannot connect to the database: {}", 0.to_string())]
    DbConnectionError(#[from] r2d2::Error),
    #[error("Database operation failed: {}", 0.to_string())]
    DbError(#[from] diesel::result::Error),
}

impl error::ResponseError for Error {
    fn error_response(&self) -> HttpResponse {
        HttpResponse::InternalServerError().json(models::ApiError {
            error: self.to_string(),
        })
    }

    fn status_code(&self) -> StatusCode {
        StatusCode::INTERNAL_SERVER_ERROR
    }
}

embed_migrations!("./migrations");

fn open_connection_pool() -> r2d2::Pool<ConnectionManager<PgConnection>> {
    let connspec = std::env::var("DATABASE_URL").expect("DATABASE_URL is not defined");

    info!("Creating db connection pool");

    let manager = ConnectionManager::<PgConnection>::new(connspec);
    r2d2::Pool::builder()
        .build(manager)
        .expect("Failed to create database connection pool")
}

#[derive(Clone)]
pub struct Store {
    pool: r2d2::Pool<ConnectionManager<PgConnection>>,
}

impl Store {
    pub fn new() -> Self {
        let store = Store {
            pool: open_connection_pool(),
        };

        store.run_migrations();

        store
    }

    fn run_migrations(&self) {
        info!("Applying migrations");

        let connection = self.open_connection().expect("Failed to open connection");
        embedded_migrations::run_with_output(&connection, &mut std::io::stdout())
            .expect("Failed to apply migrations");
    }

    fn open_connection(
        &self,
    ) -> Result<r2d2::PooledConnection<ConnectionManager<PgConnection>>, ::r2d2::Error> {
        self.pool.get()
    }

    pub fn add_product(&self, new_product: models::NewProduct) -> Result<models::Product, Error> {
        use crate::schema::products::dsl::*;
        let connection = self.open_connection()?;

        let product: models::Product = diesel::insert_into(products)
            .values(&new_product)
            .get_result(&connection)?;
        Ok(product)
    }

    pub fn get_product(&self, product_id: i32) -> Result<Option<models::Product>, Error> {
        use crate::schema::products::dsl::*;
        let connection = self.open_connection()?;

        let product = products
            .filter(id.eq(product_id))
            .first::<models::Product>(&connection)
            .optional()?;
        Ok(product)
    }

    pub fn update_product(
        &self,
        product_id: i32,
        new_product: models::NewProduct,
    ) -> Result<models::Product, Error> {
        use crate::schema::products::dsl::*;
        let connection = self.open_connection()?;

        let product = models::Product {
            id: product_id,
            code: new_product.code,
            name: new_product.name,
            category: new_product.category,
        };

        let result: models::Product = diesel::insert_into(products)
            .values(&product)
            .on_conflict(id)
            .do_update()
            .set(&product)
            .get_result(&connection)?;

        Ok(result)
    }

    pub fn remove_product(&self, product_id: i32) -> Result<usize, Error> {
        use crate::schema::products::dsl::*;
        let connection = self.open_connection()?;

        let result = diesel::delete(products)
            .filter(id.eq(product_id))
            .execute(&connection)?;

        Ok(result)
    }

    pub fn list_products(&self, query: models::ListQuery) -> Result<Vec<models::Product>, Error> {
        use crate::schema::products::dsl::*;
        let connection = self.open_connection()?;

        let result = products
            .order(id)
            .limit(query.limit.unwrap_or(std::i64::MAX))
            .offset(query.offset.unwrap_or(0i64))
            .load(&connection)?;
        Ok(result)
    }
}
