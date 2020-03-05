use diesel::prelude::*;

use crate::models;

pub fn add_product(new_product: models::NewProduct, connection: &PgConnection) -> Result<models::Product, diesel::result::Error> {
    use crate::schema::products::dsl::*;
    let product: models::Product = diesel::insert_into(products)
        .values(&new_product)
        .get_result(connection)?;
    Ok(product)
}

pub fn get_product(product_id: i32, connection: &PgConnection) -> Result<Option<models::Product>, diesel::result::Error> {
    use crate::schema::products::dsl::*;
    let product = products
        .filter(id.eq(product_id))
        .first::<models::Product>(connection)
        .optional()?;
    Ok(product)
}

pub fn update_product(product_id: i32, new_product: models::NewProduct, connection: &PgConnection) -> Result<models::Product, diesel::result::Error> {
    use crate::schema::products::dsl::*;

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
        .get_result(connection)?;

    Ok(result)
}

pub fn remove_product(product_id: i32, connection: &PgConnection) -> Result<usize, diesel::result::Error> {
    use crate::schema::products::dsl::*;

    let result = diesel::delete(products).filter(id.eq(product_id)).execute(connection)?;

    Ok(result)
}

pub fn list_products(limit: Option<i64>, offset: Option<i64>, connection: &PgConnection) -> Result<Vec<models::Product>, diesel::result::Error> {
    use crate::schema::products::dsl::*;
    let result = products
        .order(id)
        .limit(limit.unwrap_or(std::i64::MAX))
        .offset(offset.unwrap_or(0i64))
        .load(connection)?;
    Ok(result)
}
