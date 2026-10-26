use std::result;

use sqlx::migrate::MigrateError;
use thiserror::Error;

pub type Result<T> = result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
	#[error("Database error: {0}")]
	DB(#[from] sqlx::Error),
	#[error("Database migration error: {0}")]
	Migration(#[from] MigrateError),
	#[error("Invalid row: {0}")]
	Formatting(String),
}
