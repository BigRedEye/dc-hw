use {crate::models, crate::repo, errors::prelude::*};

#[derive(Clone)]
pub struct Service {
    repo: repo::PgRepo,
    auth: auth_client::client::Client,
}

impl Service {
    pub fn new(repo: repo::PgRepo, auth: auth_client::client::Client) -> Self {
        Service { repo, auth }
    }

    pub async fn auth(&self, token: String) -> Result<ServiceHandler> {
        let role = match self.auth.validate(token).await? {
            Some(role) => role,
            _ => return Err(errors::Error::Unauthorized("Permission denied".into())),
        };
        Ok(ServiceHandler { repo: self.repo.clone(), role })
    }
}

pub struct ServiceHandler {
    repo: repo::PgRepo,
    role: auth_client::Role,
}

impl ServiceHandler {
    pub fn add_product(&self, new_product: models::NewProduct) -> Result<models::Product> {
        self.assert_role(auth_client::Role::Admin)?;
        self.repo.add_product(new_product)
    }

    pub fn get_product(&self, product_id: i32) -> Result<Option<models::Product>> {
        self.assert_role(auth_client::Role::User)?;
        self.repo.get_product(product_id)
    }

    pub fn update_product(
        &self,
        product_id: i32,
        new_product: models::NewProduct,
    ) -> Result<models::Product> {
        self.assert_role(auth_client::Role::Admin)?;
        self.repo.update_product(product_id, new_product)
    }

    pub fn remove_product(&self, product_id: i32) -> Result<usize> {
        self.assert_role(auth_client::Role::Admin)?;
        self.repo.remove_product(product_id)
    }

    pub fn list_products(&self, query: models::ListQuery) -> Result<(i64, Vec<models::Product>)> {
        self.assert_role(auth_client::Role::User)?;
        self.repo.list_products(query)
    }

    fn assert_role(&self, minimal_role: auth_client::Role) -> Result<()> {
        if self.role < minimal_role {
            return Err(errors::Error::Unauthorized("Permission denied".into()))
        };
        Ok(())
    }
}
