//! Models put an abstraction between controller or "business logic" code and
//! the database itself. They hide SQL queries and include logic which is
//! intended to keep the database consistent. You should never manipulate the
//! database directly from either command-line tool or controller code.

mod attachments;
mod gh_user_records;
mod jams;
mod permissions;
mod rich_texts;

use crate::attachments::AttachmentStorageError;
pub use crate::models::{
    attachments::*, gh_user_records::*, jams::*, permissions::*, rich_texts::*,
};
use diesel::{r2d2::PoolError, result::Error as DieselError};
use std::{path::PathBuf, convert::TryInto};
use thiserror::Error;

/// An error common to model helper functions.
#[derive(Error, Debug)]
pub enum ModelError {
    /// Failed to get a database connection.
    #[error("Couldn't get out of the pool with error {0}. Send a lifeguard.")]
    PoolError(#[from] PoolError),

    /// Failed to query the database, or no result from the database when one
    /// was expected.
    #[error("Couldn't query the database with error {0}. Send a DBA.")]
    DieselError(#[from] DieselError),

    #[error("The file {0} could not be found. Send in Search and Rescue")]
    FileNotFoundError(PathBuf),

    #[error("Could not store attachment with error {0}")]
    AttachmentStorageError(#[from] AttachmentStorageError),
}

// Gets the most recently inserted row. Please only use this from within a
// transaction to avoid threading adventures.
no_arg_sql_function!(
    LastInsertRowid,
    diesel::sql_types::Integer,
    "Represents the SQL last_insert_row() function"
);

/// Converts a diesel result, which packages the absence of a record as an
/// error, into an Option, which makes dealing with "I'm okay with something not
/// being present" slightly more Rustic.
fn r_to_opt<T>(
    r: Result<T, diesel::result::Error>,
) -> Result<Option<T>, ModelError> {
    match r {
        Ok(t) => Ok(Some(t)),
        Err(diesel::NotFound) => Ok(None),
        Err(e) => Err(ModelError::DieselError(e)),
    }
}

pub enum ApprovalState {
    Draft = 0, Submitted = 2, Approved = 4, Rejected = 8
}

pub enum ApprovalStateDeserializationError {
    BadValue(i32)
}

impl TryInto<ApprovalState> for i32 {
    type Error = ApprovalStateDeserializationError;

    fn try_into(self) -> Result<ApprovalState, Self::Error> {
        match self {
            0 => Ok(ApprovalState::Draft),
            2 => Ok(ApprovalState::Submitted),
            4 => Ok(ApprovalState::Approved),
            8 => Ok(ApprovalState::Rejected),
            val => Err(ApprovalStateDeserializationError::BadValue(val))
        }
    }
}
