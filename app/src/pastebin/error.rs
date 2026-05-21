use thiserror::Error;
use tokio::io;

use crate::storage;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
	#[error("Tcp error: {0}")]
	TcpErr(#[from] io::Error),
	#[error("Storage error: {0}")]
	StorageErr(#[from] storage::Error),
}
