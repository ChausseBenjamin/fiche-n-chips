use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
	#[error("Database error: {0}")]
	Db(#[from] turso::Error),
	#[error("Failed to read file: {0}")]
	Io(#[from] std::io::Error),
	#[error("Initialization failure: {0}")]
	Setup(String),
	#[error("Parsing error: {0}")]
	Parse(String),
}
