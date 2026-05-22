use thiserror::Error;

use crate::storage;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
	#[error("Tcp error: {0}")]
	TcpErr(#[from] tokio::io::Error),
	#[error("Storage error: {0}")]
	StorageErr(#[from] storage::Error),
	#[error("Compression error: {0}")]
	CompError(#[from] std::io::Error),
}
