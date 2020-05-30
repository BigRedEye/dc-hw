use std::time::SystemTime;
use serde::{Deserialize, Serialize};
use crate::schema::*;

#[derive(Serialize, Deserialize, Debug, Copy, Clone, DbEnum)]
#[DieselType = "Access_level"]
pub enum AccessLevel {
    Admin,
    User,
}

impl std::default::Default for AccessLevel {
    fn default() -> Self {
        AccessLevel::User
    }
}

pub enum Login {
    Email(String),
    Phone(String),
}

#[derive(Serialize, Deserialize, Queryable, Identifiable, Debug)]
#[table_name = "users"]
pub struct User {
    pub id: i32,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub password: String,
    pub permissions: AccessLevel,
}

#[derive(Insertable, Serialize, Deserialize, Queryable, Clone)]
#[table_name = "users"]
pub struct NewUser {
    pub phone: Option<String>,
    pub email: Option<String>,
    pub password: String,

    #[serde(skip_deserializing)]
    pub permissions: AccessLevel,
}

#[derive(Queryable, Identifiable)]
#[table_name = "sessions"]
pub struct Session {
    pub id: i32,
    pub refresh_token: String,
    pub access_token: String,
    pub expires_at: SystemTime,
    pub user_id: i32,
}

#[derive(Insertable)]
#[table_name = "sessions"]
pub struct NewSession<'a> {
    pub refresh_token: &'a str,
    pub access_token: &'a str,
    pub expires_at: SystemTime,
    pub user_id: i32,
}

#[derive(Queryable, Identifiable)]
#[table_name = "confirmations"]
pub struct Confirmation {
    pub id: i32,
    pub token: String,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub user_id: i32,
}

#[derive(Insertable)]
#[table_name = "confirmations"]
pub struct NewConfirmation {
    pub token: String,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub user_id: i32,
}

#[derive(Deserialize)]
pub struct LoginRequest {
    pub login: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct LoginResponse {
    pub access_token: String,
    pub refresh_token: String,
}

#[derive(Serialize)]
pub struct RefreshRequest {
    pub refresh_token: String,
}

#[derive(Serialize)]
pub struct UpdateUserRequest {
    pub user_id: String,
    pub role: AccessLevel,
}

#[derive(Serialize)]
pub struct ValidateTokenResponse {
    pub valid: bool,
    pub role: AccessLevel,
}

pub struct ListUsersRequest {
    pub offset: Option<i64>,
    pub limit: Option<i64>,
}

pub struct ListUsersResponse {
    pub users: Vec<User>,
}
