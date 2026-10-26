use std::convert::Infallible;
use std::io;

use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
	#[error("Parsing error: {0}")]
	Parse(String),
	#[error("Codec error: {0}")]
	Codec(#[from] io::Error),
}
