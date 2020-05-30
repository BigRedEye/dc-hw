use thiserror::Error;

use diesel;
use r2d2;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Could not connect to the database: {}", .0.to_string())]
    DbConnection(#[from] r2d2::Error),

    #[error("Database migration failed: {}", .0.to_string())]
    DbMigration(#[from] diesel_migrations::RunMigrationsError),

    #[error("Database operation failed: {}", .0.to_string())]
    DbInternal(#[source] diesel::result::Error),

    #[error("Database query failed: {}", .0.to_string())]
    DbNotFound(#[source] diesel::result::Error),

    #[error("Database query failed: {}", .0.to_string())]
    DbNonUnique(#[source] diesel::result::Error),

    #[error("Internal error")]
    BcryptError(#[from] bcrypt::BcryptError),

    #[error("gRPC connection error: {}", .0.to_string())]
    GrpcConnection(#[from] tonic::transport::Error),

    #[error("gRPC call error: {}", .0.to_string())]
    GrpcError(#[from] tonic::Status),

    #[error("Amqp error: {}", .0.to_string())]
    AmqpError(#[from] lapin::Error),

    #[error("Protobuf parsing failed: {}", .0.to_string())]
    ProtoDecodeError(#[from] prost::DecodeError),

    #[error("Protobuf encoding failed: {}", .0.to_string())]
    ProtoEncodeError(#[from] prost::EncodeError),

    #[error("Internal error")]
    Internal(#[from] anyhow::Error),

    #[error("Bad request: {}", .0)]
    BadRequest(String),

    #[error("Not found: {}", .0)]
    NotFound(String),

    #[error("Unauthorized: {}", .0)]
    Unauthorized(String),
}

impl From<diesel::result::Error> for Error {
    fn from(error: diesel::result::Error) -> Error {
        match error {
            diesel::result::Error::NotFound => Error::DbNotFound(error),
            diesel::result::Error::DatabaseError(diesel::result::DatabaseErrorKind::UniqueViolation, _) => Error::DbNonUnique(error),
            other => Error::DbInternal(other),
        }
    }
}

impl From<Error> for tonic::Status {
    fn from(error: Error) -> tonic::Status {
        match error {
            Error::BadRequest(x) => tonic::Status::invalid_argument(x),
            Error::NotFound(x) => tonic::Status::not_found(x),
            Error::Unauthorized(x) => tonic::Status::unauthenticated(x),
            Error::Internal(x) => tonic::Status::internal(x.to_string()),
            x => tonic::Status::internal(x.to_string())
        }
    }
}
