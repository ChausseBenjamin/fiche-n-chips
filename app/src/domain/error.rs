use std::convert::Infallible;
use std::io;

use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Error)]
pub enum Error {
	#[error("An impossible error just created an error... WTF?!!")]
	None(#[from] Infallible),
	#[error("Parsing error: {0}")]
	Parse(String),
	#[error("Codec error: {0}")]
	Codec(#[from] io::Error),
}
