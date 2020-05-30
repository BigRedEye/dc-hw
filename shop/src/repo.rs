use log::info;

use diesel::prelude::*;
use diesel::r2d2::ConnectionManager;

use errors::prelude::*;
use crate::models;
use crate::config;

type ConnectionPool = diesel::r2d2::Pool<ConnectionManager<PgConnection>>;
type Connection = diesel::r2d2::PooledConnection<ConnectionManager<PgConnection>>;

#[derive(Clone)]
pub struct PgRepo {
    pool: ConnectionPool,
}

embed_migrations!("./migrations");

fn open_connection_pool(cfg: &config::Settings) -> Result<ConnectionPool> {
    info!("Creating db connection pool at {}", cfg.database_url);

    let manager = ConnectionManager::<PgConnection>::new(&cfg.database_url);
    let pool = diesel::r2d2::Pool::builder().build(manager)?;
    Ok(pool)
}

impl PgRepo {
    pub fn new(cfg: &config::Settings) -> Result<PgRepo> {
        let pool = open_connection_pool(cfg)?;
        let repo = PgRepo{ pool };
        repo.run_migrations()?;
        Ok(repo)
    }

    fn run_migrations(&self) -> Result<()> {
        info!("Applying migrations");

        let connection = self.open_connection()?;
        embedded_migrations::run_with_output(&connection, &mut std::io::stdout())?;

        Ok(())
    }

    fn open_connection(&self) -> Result<Connection> {
        self.pool.get().map_err(errors::Error::DbConnection)
    }

    pub fn add_product(&self, new_product: models::NewProduct) -> Result<models::Product> {
        use crate::schema::products::dsl::*;
        let connection = self.open_connection()?;

        let product: models::Product = diesel::insert_into(products)
            .values(&new_product)
            .get_result(&connection)?;

        Ok(product)
    }

    pub fn add_products(&self, new_products: &[models::NewProduct]) -> Result<()> {
        use crate::schema::products::dsl::*;
        let connection = self.open_connection()?;

        diesel::insert_into(products)
            .values(new_products)
            .on_conflict_do_nothing()
            .execute(&connection)?;

        Ok(())
    }

    pub fn get_product(&self, product_id: i32) -> Result<Option<models::Product>> {
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
    ) -> Result<models::Product> {
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

    pub fn remove_product(&self, product_id: i32) -> Result<usize> {
        use crate::schema::products::dsl::*;
        let connection = self.open_connection()?;

        let result = diesel::delete(products)
            .filter(id.eq(product_id))
            .execute(&connection)?;

        Ok(result)
    }

    pub fn list_products(&self, query: models::ListQuery) -> Result<(i64, Vec<models::Product>)> {
        use crate::schema::products::dsl::*;
        let connection = self.open_connection()?;

        let result = products
            .order(id)
            .limit(query.limit.unwrap_or(std::i64::MAX))
            .offset(query.offset.unwrap_or(0i64))
            .load(&connection)?;

        let count = products
            .select(diesel::dsl::count_star())
            .first(&connection)?;

        Ok((count, result))
    }
}
