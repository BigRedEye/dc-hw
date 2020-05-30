use tonic::{Request, Response, Status};

use errors::Error;
use errors::prelude::*;
use crate::service;
use pb::shop_server::Shop;

pub struct Server {
    shop: service::Service
}

fn parse_token<T>(request: &Request<T>) -> Result<String> {
    let token = match request.metadata().get("authorization") {
        Some(token) => token.to_str().map_err(|_| Error::Unauthorized("Invalid token value".into()))?,
        None => return Err(Error::Unauthorized("Token not found".into())),
    };

    let token = token.trim_start_matches("Bearer ");
    Ok(token.into())
}

impl Server {
    pub fn new(shop: service::Service) -> Server {
        Server { shop }
    }
}

#[tonic::async_trait]
impl Shop for Server {
    async fn add_product(
        &self,
        request: Request<pb::AddProductRequest>,
    ) -> std::result::Result<Response<pb::AddProductResponse>, Status> {
        let token = parse_token(&request)?;
        let product = self.shop
            .auth(token).await?
            .add_product(request.into_inner().product.into())?;
        Ok(Response::new(pb::AddProductResponse{ product: product.into() }))
    }

    async fn update_product(
        &self,
        request: Request<pb::UpdateProductRequest>,
    ) -> std::result::Result<Response<pb::UpdateProductResponse>, Status> {
        let token = parse_token(&request)?;
        let id = request.get_ref().product.id.unwrap_or(0);
        let product = self.shop
            .auth(token).await?
            .update_product(id, request.into_inner().product.into())?;
        Ok(Response::new(pb::UpdateProductResponse{ product: product.into() }))
    }

    async fn get_product(
        &self,
        request: Request<pb::GetProductRequest>,
    ) -> std::result::Result<Response<pb::GetProductResponse>, Status> {
        let token = parse_token(&request)?;
        let id = request.get_ref().id;
        let product = self.shop
            .auth(token).await?
            .get_product(id)?
            .ok_or(errors::Error::NotFound("Product not found".into()))?;
        Ok(Response::new(pb::GetProductResponse{ product: product.into() }))
    }

    async fn delete_product(
        &self,
        request: Request<pb::DeleteProductRequest>,
    ) -> std::result::Result<Response<pb::DeleteProductResponse>, Status> {
        let token = parse_token(&request)?;
        let id = request.get_ref().id;
        let cnt = self.shop
            .auth(token).await?
            .remove_product(id)?;
        match cnt {
            0 => Err(errors::Error::NotFound("Product not found".into()).into()),
            _ => Ok(Response::new(pb::DeleteProductResponse::default()))
        }
    }

    async fn list_products(
        &self,
        request: Request<pb::ListProductsRequest>,
    ) -> std::result::Result<Response<pb::ListProductsResponse>, Status> {
        let token = parse_token(&request)?;
        let (cnt, res) = self.shop
            .auth(token).await?
            .list_products(request.into_inner().into())?;
        Ok(Response::new(pb::ListProductsResponse{ count: cnt, products: res.into_iter().map(|p| p.into()).collect() }))
    }
}

