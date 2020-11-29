pub mod attachments;
pub mod gh_oauth;
pub mod homepage;
pub mod jam_entries;
pub mod jams;

use rocket::{
    http::Status, response::Responder, response::Result as RocketResult,
    Request,
};
use thiserror::Error;

/// Unified error type for most (all?) handlers. Puts all the annoying
/// boilerplate of derives into one spot with a single implementation of
/// Responder to map these back onto error codes that ought to go back and hit
/// the right error catchers.
///
/// Note that it would be very tempting to use anyhow for this, however we
/// cannot implement Responder for it. Even if we could, inferring the http
/// status code from a Boxed error would be rather challenging.
#[derive(Debug, Error)]
pub enum HandlerError {
    #[error("The resource was not found")]
    NotFound,

    #[error("Could not get a connection from the pool with error {0}")]
    PoolError(#[from] diesel::r2d2::PoolError),

    #[error("Failed to query the database with error {0}")]
    DatabaseError(#[from] crate::models::ModelError),

    #[error("Failed to store/retrieve attachment with error {0}")]
    AttachmentStorageError(#[from] crate::attachments::AttachmentStorageError),

    #[error("HTTP Error {0}")]
    HttpError(#[from] reqwest::Error),

    #[error("Parse Error {0}")]
    ParseError(#[from] chrono::ParseError),

    #[error("Diesel Error {0}")]
    DieselError(#[from] diesel::result::Error),

    #[error("Approval State Parse Error {0}")]
    ApprovalStateParseError(#[from] crate::models::ApprovalStateParseError),
}

impl<'r, 'o: 'r> Responder<'r, 'o> for HandlerError {
    fn respond_to(self, _request: &'r Request<'_>) -> RocketResult<'o> {
        let r = match self {
            HandlerError::AttachmentStorageError(_) => {
                Status::InternalServerError
            }
            HandlerError::DatabaseError(_) => Status::InternalServerError,
            HandlerError::PoolError(_) => Status::InternalServerError,
            HandlerError::HttpError(_) => Status::InternalServerError,
            HandlerError::ParseError(_) => Status::InternalServerError,
            HandlerError::ApprovalStateParseError(_) => {
                Status::InternalServerError
            }
            HandlerError::DieselError(_) => Status::InternalServerError,
            HandlerError::NotFound => Status::NotFound,
        };

        if r == Status::InternalServerError {
            print!("Internal error {:?}", self);
        }

        Err(r)
    }
}
