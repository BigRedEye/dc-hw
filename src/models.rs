use serde::{Deserialize, Serialize};

use crate::schema::products;

#[derive(Serialize, Queryable, Insertable, AsChangeset)]
pub struct Product {
    pub id: i32,
    pub name: String,
    pub code: String,
    pub category: String,
}

#[derive(Serialize, Deserialize, Insertable)]
#[table_name="products"]
pub struct NewProduct {
    pub name: String,
    pub code: String,
    pub category: String,
}

#[derive(Serialize)]
pub struct ApiError {
    pub error: String,
}

#[derive(Deserialize)]
pub struct ListQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

