use tonic::{Request, Response, Status};

use errors::Error;
use errors::prelude::*;
use crate::service;
use crate::models;
use pb::auth_server::Auth;

pub struct Server {
    auth: service::Service,
    auth_client: auth_client::client::Client,
}

impl Server {
    pub fn new(auth: service::Service, auth_client: auth_client::client::Client) -> Server {
        Server { auth, auth_client }
    }
}

fn parse_token<T>(request: &Request<T>) -> Result<String> {
    let token = match request.metadata().get("authorization") {
        Some(token) => token.to_str().map_err(|_| Error::Unauthorized("Invalid token value".into()))?,
        None => return Err(Error::Unauthorized("No access token found".into())),
    };

    let token = token.trim_start_matches("Bearer ");
    Ok(token.into())
}

#[tonic::async_trait]
impl Auth for Server {
    async fn register(
        &self,
        request: Request<pb::RegisterRequest>,
    ) -> std::result::Result<Response<pb::RegisterResponse>, Status> {
        self.auth.register(request.into_inner().into()).await?;
        Ok(Response::new(pb::RegisterResponse::default()))
    }

    async fn login(
        &self,
        request: Request<pb::LoginRequest>,
    ) -> std::result::Result<Response<pb::LoginResponse>, Status> {
        let response = self.auth.login(request.into_inner().into())?;
        Ok(Response::new(response.into()))
    }

    async fn confirm(
        &self,
        request: Request<pb::ConfirmRequest>,
    ) -> std::result::Result<Response<pb::ConfirmResponse>, Status> {
        self.auth.confirm(&request.into_inner().token)?;
        Ok(Response::new(pb::ConfirmResponse::default()))
    }

    async fn refresh(
        &self,
        request: Request<pb::RefreshRequest>,
    ) -> std::result::Result<Response<pb::RefreshResponse>, Status> {
        let response = self.auth.refresh(models::RefreshRequest{
            refresh_token: request.into_inner().token
        })?;
        Ok(Response::new(response.into()))
    }

    async fn list_users(
        &self,
        request: Request<pb::ListUsersRequest>,
    ) -> std::result::Result<Response<pb::ListUsersResponse>, Status> {
        self.auth_client.validate_role(parse_token(&request)?, auth_client::Role::Admin).await?;
        let users = self.auth.list_users(request.into_inner().into())?;
        Ok(Response::new(users.into()))
    }

    async fn update_user(
        &self,
        request: Request<pb::UpdateUserRequest>,
    ) -> std::result::Result<Response<pb::UpdateUserResponse>, Status> {
        self.auth_client.validate_role(parse_token(&request)?, auth_client::Role::Admin).await?;
        self.auth.set_user_role(request.into_inner().into())?;
        Ok(Response::new(pb::UpdateUserResponse::default()))
    }

    async fn validate_token(
        &self,
        request: Request<pb::ValidateTokenRequest>,
    ) -> std::result::Result<Response<pb::ValidateTokenResponse>, Status> {
        let res = self.auth.validate(&request.into_inner().token);
        Ok(Response::new(res.into()))
    }
}

