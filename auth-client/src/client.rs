use pb::auth_client::AuthClient;
use errors::prelude::*;
use log::info;

#[derive(Clone)]
pub struct Client {
    endpoint: tonic::transport::Endpoint
}

impl Client {
    pub fn new(endpoint: &str) -> Result<Client> {
        let endpoint = format!("http://{}", endpoint);
        info!("Creating auth client for endpoint {}", endpoint);
        let endpoint = tonic::transport::Endpoint::from_shared(endpoint.to_string())
            .map_err(|e| errors::Error::Internal(e.into()))?;
        Ok(Client { endpoint })
    }

    pub async fn validate(&self, token: String) -> Result<Option<crate::Role>> {
        let mut client = AuthClient::connect(self.endpoint.clone()).await?;
        
        let request = tonic::Request::new(pb::ValidateTokenRequest { token });
        let response = client.validate_token(request).await?;
        let message = response.into_inner();

        if !message.valid {
            return Ok(None);
        }

        Ok(pb::Role::from_i32(message.role))
    }

    pub async fn validate_role(&self, token: String, role: crate::Role) -> Result<()> {
        let res = self.validate(token).await?;
        match res {
            Some(user_role) if user_role == role => Ok(()),
            _ => Err(errors::Error::Unauthorized("Permission denied".into())),
        }
    }
}
