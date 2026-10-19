use actix_web::http::StatusCode;
use awc::ClientResponse;
use awc::error::SendRequestError;
use thiserror::Error;

use crate::storage;

#[derive(Debug, Error)]
pub enum Error {
	#[error("notify url not configured")]
	Unset,
	#[error("database error: {0}")]
	Storage(#[from] storage::Error),
	#[error("failed to emit alert: {0}")]
	Emit(String),
	#[error("ntfy returned {status}: {body}")]
	NtfyError { status: StatusCode, body: String },
	#[error("{0} isn't a valid content kind")]
	UnknownKind(&'static str),
}

impl Error {
	pub async fn from_response(mut resp: ClientResponse) -> Self {
		let status = resp.status();
		let body = match resp.body().await {
			Ok(bytes) => String::from_utf8_lossy(&bytes).into_owned(),
			Err(e) => format!("<failed to read body: {e}"),
		};
		Self::NtfyError { status, body }
	}
}

impl From<SendRequestError> for Error {
	fn from(e: SendRequestError) -> Self { Self::Emit(e.to_string()) }
}
