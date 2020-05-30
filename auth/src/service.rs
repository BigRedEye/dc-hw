use {
    std::time::{Duration, SystemTime},

    errors::Error,
    errors::prelude::*,

    crate::models,
    crate::repo,
    crate::confirms,
    crate::config,

    rand::prelude::*,
    rand::distributions::Alphanumeric,

    async_trait::async_trait,

    bcrypt,
};

pub trait UsersRepo {
    fn add_user(&self, user: models::NewUser) -> Result<models::User>;
    fn get_user_by_login(&self, login: &str) -> Result<models::User>;

    fn confirm_user(&self, user: i32, login: models::Login) -> Result<()>;
    fn set_user_role(&self, user: i32, role: models::AccessLevel) -> Result<()>;
    fn get_user_role(&self, user: i32) -> Result<models::AccessLevel>;
    fn get_password_hash(&self, login: &str) -> Result<String>;
    fn list_users(&self, req: models::ListUsersRequest) -> Result<models::ListUsersResponse>;
}

pub trait TokensRepo {
    fn add_session(&self, session: models::NewSession) -> Result<()>;
    fn get_session_by_access_token(&self, token: &str) -> Result<models::Session>;
    fn get_session_by_refresh_token(&self, token: &str) -> Result<models::Session>;
    fn remove_session(&self, id: i32) -> Result<usize>;
}

pub trait ConfirmationsRepo {
    fn add_confirmation(&self, user: i32, login: &models::Login, token: &str) -> Result<()>;
    fn find_confirmation(&self, token: &str) -> Result<models::Confirmation>;
    fn remove_confirmation(&self, token: &str) -> Result<()>;
}

#[async_trait]
pub trait ConfirmationsSender {
    async fn send(&self, login: models::Login, token: String) -> Result<()>;
}

#[derive(Clone)]
pub struct Service<> {
    session_timeout: u32,
    repo: repo::PgRepo,
    confirms_sender: confirms::ConfrimsSender,
}

const BCRYPT_COST: u32 = 10;

impl Service {
    pub fn new(cfg: &config::Settings, repo: repo::PgRepo, confirms_sender: confirms::ConfrimsSender) -> Self {
        Service {
            session_timeout: cfg.session_timeout,
            repo,
            confirms_sender
        }
    }

    pub async fn register(&self, user: models::NewUser) -> Result<()> {
        if user.email.is_none() && user.phone.is_none() {
            return Err(Error::BadRequest("Login is required".into()));
        }
        if user.password.is_empty() {
            return Err(Error::BadRequest("Password is required".into()));
        }

        let user_id = self.register_user(user.clone())?;
        match self.generate_confirmations(user_id, &user).await {
            Ok(_) => (),
            Err(Error::DbNonUnique(_)) => return Err(Error::BadRequest("Login is already used".into())),
            Err(e) => return Err(e),
        }

        Ok(())
    }

    fn register_user(&self, mut user: models::NewUser) -> Result<i32> {
        user.password = Self::hash_password(user.password)?;
        user.email = None;
        user.phone = None;
        let user = self.repo.add_user(user)?;
        Ok(user.id)
    }

    fn hash_password(password: String) -> Result<String> {
        let hash = bcrypt::hash(password, BCRYPT_COST)?;
        Ok(hash)
    }

    async fn generate_confirmations(&self, user_id: i32, user: &models::NewUser) -> Result<()> {
        if let Some(login) = user.email.as_ref() {
            self.generate_confirmation(models::Login::Email(login.clone()), user_id).await?;
        }
        if let Some(login) = user.phone.as_ref() {
            self.generate_confirmation(models::Login::Phone(login.clone()), user_id).await?;
        }

        Ok(())
    }

    fn gen_token() -> String {
        rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(32)
            .collect::<String>()
    }

    async fn generate_confirmation(&self, login: models::Login, user: i32) -> Result<()> {
        let token = Self::gen_token();

        self.repo.add_confirmation(user, &login, &token)?;
        self.confirms_sender.send(login, token).await?;

        Ok(())
    }

    pub fn login(&self, request: models::LoginRequest) -> Result<models::LoginResponse> {
        let user = match self.repo.get_user_by_login(&request.login) {
            Ok(user) => user,
            Err(Error::DbNotFound(_)) => return Err(Error::Unauthorized("Invalid credentials".into())),
            Err(e) => return Err(e)
        };
        let hash_equal = bcrypt::verify(request.password, &user.password)?;
        if hash_equal {
            Ok(self.gen_tokens(user.id)?)
        } else {
            Err(Error::Unauthorized("Invalid credentials".into()))
        }
    }

    fn gen_tokens(&self, user_id: i32) -> Result<models::LoginResponse> {
        let access_token = Self::gen_token();
        let refresh_token = Self::gen_token();

        let session = models::NewSession {
            refresh_token: &refresh_token,
            access_token: &access_token,
            expires_at: std::time::SystemTime::now() + Duration::new(self.session_timeout.into(), 0),
            user_id,
        };
        self.repo.add_session(session)?;

        let res = models::LoginResponse {
            access_token,
            refresh_token,
        };
        Ok(res)
    }

    pub fn validate(&self, token: &str) -> models::ValidateTokenResponse {
        let session = match self.repo.get_session_by_access_token(token) {
            Ok(s) => s,
            Err(_) => return models::ValidateTokenResponse{ valid: false, role: models::AccessLevel::User },
        };

        let mut valid = SystemTime::now() < session.expires_at;
        let role = self.repo.get_user_role(session.user_id).unwrap_or_else(|_| {
            valid = false;
            models::AccessLevel::User
        });

        models::ValidateTokenResponse{ valid, role }
    }

    pub fn refresh(&self, req: models::RefreshRequest) -> Result<models::LoginResponse> {
        let session = match self.repo.get_session_by_refresh_token(&req.refresh_token) {
            Ok(s) => s,
            Err(Error::DbNotFound(_)) => return Err(Error::Unauthorized("Unknown session".into())),
            Err(error) => return Err(error.into()),
        };

        let _ = self.repo.remove_session(session.id);

        Ok(self.gen_tokens(session.user_id)?)
    }

    pub fn confirm(&self, token: &str) -> Result<()> {
        let confirmation = self.repo.find_confirmation(token)?;
        self.repo.remove_confirmation(token)?;
        if let Some(email) = confirmation.email {
            self.repo.confirm_user(confirmation.user_id, models::Login::Email(email))?;
        }
        if let Some(phone) = confirmation.phone {
            self.repo.confirm_user(confirmation.user_id, models::Login::Email(phone))?;
        }
        Ok(())
    }

    pub fn set_user_role(&self, req: models::UpdateUserRequest) -> Result<()> {
        self.repo.set_user_role(req.user_id.parse().map_err(|_| Error::BadRequest("Failed to parse user id".into()))?, req.role)
    }

    pub fn list_users(&self, req: models::ListUsersRequest) -> Result<models::ListUsersResponse> {
        self.repo.list_users(req)
    }
}
