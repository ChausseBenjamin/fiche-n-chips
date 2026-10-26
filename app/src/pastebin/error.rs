use std::result;

use thiserror::Error;

use crate::storage;

pub type Result<T> = result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
	#[error("I/O error: {0}")]
	IoErr(#[from] std::io::Error),
	#[error("Storage error: {0}")]
	StorageErr(#[from] storage::Error),
}
