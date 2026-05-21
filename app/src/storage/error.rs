use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
	#[error("Database error: {0}")]
	DbErr(#[from] turso::Error),
	#[error("Failed to read file: {0}")]
	IoErr(#[from] std::io::Error),
	#[error("Initialization failure: {0}")]
	SetupErr(String),
	#[error("Parsing error: {0}")]
	ParseErr(String),
}
