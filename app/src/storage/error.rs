use std::result;

use sqlx::migrate::MigrateError;
use thiserror::Error;

use crate::domain::Slug;

pub type Result<T> = result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
	#[error("slug `{0}` is already taken")]
	SlugTaken(Slug),
	#[error("no free slug after {0} attempts")]
	Exhausted(usize),
	#[error("Database error: {0}")]
	DB(#[from] sqlx::Error),
	#[error("Database migration error: {0}")]
	Migration(#[from] MigrateError),
	#[error("Invalid row: {0}")]
	Formatting(String),
}
