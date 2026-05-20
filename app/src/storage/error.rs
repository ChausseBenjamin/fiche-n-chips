use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
	#[error("Database error: {0}")]
	DbError(#[from] turso::Error),
	#[error("Failed to read file: {0}")]
	IoError(#[from] std::io::Error),
	#[error("Initialization failure: {0}")]
	SetupError(String),
}
