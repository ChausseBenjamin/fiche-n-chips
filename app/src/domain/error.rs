use std::io;

use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
	#[error("URL exceeds maximum length of {max} bytes (got {actual})")]
	UrlTooLong { max: u64, actual: u64 },
	#[error("invalid slug: {reason}")]
	InvalidSlug { reason: String },
	#[error("Parsing error: {0}")]
	Parse(String),
	#[error("Codec error: {0}")]
	Codec(#[from] io::Error),
}
