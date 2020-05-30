use log::info;

use diesel::prelude::*;
use diesel::r2d2::ConnectionManager;

use errors::prelude::*;
use crate::service;
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
}

impl service::UsersRepo for PgRepo {
    fn add_user(&self, user: models::NewUser) -> Result<models::User> {
        use crate::schema::users::dsl::*;
        let connection = self.open_connection()?;

        let user = diesel::insert_into(users)
            .values(user)
            .get_result(&connection)?;

        Ok(user)
    }

    fn get_user_by_login(&self, login: &str) -> Result<models::User> {
        use crate::schema::users::dsl::*;
        let connection = self.open_connection()?;

        let user = users
            .filter(email.eq(login))
            .or_filter(phone.eq(login))
            .get_result(&connection)?;

        Ok(user)
    }

    fn confirm_user(&self, user: i32, login: models::Login) -> Result<()> {
        use crate::schema::users::dsl::*;
        let connection = self.open_connection()?;

        let req = diesel::update(users.filter(id.eq(user)));
        let req = match login {
            models::Login::Email(new_email) => req.set(email.eq(new_email)),
            models::Login::Phone(new_phone) => req.set(email.eq(new_phone)),
        };
        req.execute(&connection)?;

        Ok(())
    }

    fn set_user_role(&self, user: i32, role: models::AccessLevel) -> Result<()> {
        use crate::schema::users::dsl::*;
        let connection = self.open_connection()?;

        let req = diesel::update(users.filter(id.eq(user))).set(permissions.eq(role));
        req.execute(&connection)?;

        Ok(())
    }

    fn get_user_role(&self, user: i32) -> Result<models::AccessLevel> {
        use crate::schema::users::dsl::*;
        let connection = self.open_connection()?;

        let perms = users
            .filter(id.eq(user))
            .select(permissions)
            .get_result(&connection)?;

        Ok(perms)
    }

    fn get_password_hash(&self, login: &str) -> Result<String> {
        use crate::schema::users::dsl::*;
        let connection = self.open_connection()?;

        let hash: String = users
            .filter(email.eq(login))
            .or_filter(phone.eq(login))
            .select(password)
            .get_result(&connection)?;

        Ok(hash)
    }

    fn list_users(&self, req: models::ListUsersRequest) -> Result<models::ListUsersResponse> {
        use crate::schema::users::dsl::*;
        let connection = self.open_connection()?;

        let res = users
            .offset(req.offset.unwrap_or(0))
            .limit(req.limit.unwrap_or(i64::max_value()))
            .load(&connection)?;

        Ok(models::ListUsersResponse{ users: res })
    }
}

impl service::TokensRepo for PgRepo {
    fn add_session(&self, session: models::NewSession) -> Result<()> {
        use crate::schema::sessions::dsl::*;
        let connection = self.open_connection()?;

        diesel::insert_into(sessions)
            .values(session)
            .execute(&connection)?;

        Ok(())
    }

    fn remove_session(&self, session_id: i32) -> Result<usize> {
        use crate::schema::sessions::dsl::*;
        let connection = self.open_connection()?;

        let count = diesel::delete(sessions)
            .filter(id.eq(session_id))
            .execute(&connection)?;

        Ok(count)
    }

    fn get_session_by_access_token(&self, token: &str) -> Result<models::Session> {
        use crate::schema::sessions::dsl::*;
        let connection = self.open_connection()?;

        let session = sessions
            .filter(access_token.eq(token))
            .get_result(&connection)?;

        Ok(session)
    }

    fn get_session_by_refresh_token(&self, token: &str) -> Result<models::Session> {
        use crate::schema::sessions::dsl::*;
        let connection = self.open_connection()?;

        let session = sessions
            .filter(refresh_token.eq(token))
            .get_result(&connection)?;

        Ok(session)
    }
}

impl service::ConfirmationsRepo for PgRepo {
    fn add_confirmation(&self, user: i32, login: &models::Login, confirmation_token: &str) -> Result<()> {
        use crate::schema::confirmations::dsl::*;
        let connection = self.open_connection()?;

        let mut value = models::NewConfirmation {
            token: confirmation_token.to_owned(),
            phone: None,
            email: None,
            user_id: user,
        };
        match login {
            models::Login::Email(new_email) => value.email = Some(new_email.to_owned()),
            models::Login::Phone(new_phone) => value.phone = Some(new_phone.to_owned()),
        };

        diesel::insert_into(confirmations)
            .values(value)
            .execute(&connection)?;

        Ok(())
    }

    fn find_confirmation(&self, confirmation_token: &str) -> Result<models::Confirmation> {
        use crate::schema::confirmations::dsl::*;
        let connection = self.open_connection()?;

        let confirmation = confirmations
            .filter(token.eq(confirmation_token))
            .get_result(&connection)?;

        Ok(confirmation)
    }

    fn remove_confirmation(&self, confirmation_token: &str) -> Result<()> {
        use crate::schema::confirmations::dsl::*;
        let connection = self.open_connection()?;

        diesel::delete(confirmations)
            .filter(token.eq(confirmation_token))
            .execute(&connection)?;

        Ok(())
    }
}
