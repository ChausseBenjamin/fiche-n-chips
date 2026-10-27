use std::result;

use sqlx::migrate::MigrateError;
use thiserror::Error;

pub type Result<T> = result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
	#[error("Validation error: {0}")]
	Validation(String),
	#[error("Database error: {0}")]
	DB(#[from] sqlx::Error),
	#[error("Could not find a unique slug name after {0} attempts")]
	Collision(usize),
	#[error("Database migration error: {0}")]
	Migration(#[from] MigrateError),
	#[error("Invalid row: {0}")]
	Formatting(String),
}
