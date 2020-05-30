use crate::models;

impl From<pb::AddProductRequest> for models::NewProduct {
    fn from(req: pb::AddProductRequest) -> models::NewProduct {
        return models::NewProduct {
            code: req.product.code,
            name: req.product.name,
            category: req.product.category,
        }
    }
}

impl From<models::Product> for pb::Product {
    fn from(res: models::Product) -> pb::Product {
        return pb::Product {
            id: Some(res.id),
            code: res.code,
            name: res.name,
            category: res.category,
        }
    }
}

impl From<pb::Product> for models::NewProduct {
    fn from(res: pb::Product) -> models::NewProduct {
        return models::NewProduct {
            code: res.code,
            name: res.name,
            category: res.category,
        }
    }
}

impl From<pb::ListProductsRequest> for models::ListQuery {
    fn from(req: pb::ListProductsRequest) -> models::ListQuery {
        return models::ListQuery {
            limit: req.limit,
            offset: req.offset,
        }
    }
}
