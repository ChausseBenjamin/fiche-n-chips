use std::time::SystemTime;

use super::Route;

/// Representation where the content remains compressed
pub struct CompRedirect {
	pub(super) short: String,
	pub(super) destination: Vec<u8>,
	pub(super) created: SystemTime,
}

impl From<CompRedirect> for Route {
	fn from(value: CompRedirect) -> Self {
		Route::Redirect(value)
	}
}

/// Representation where the content hasn't been compressed
pub struct RawRedirect {
	short: String, // shortened version of the urls chosen by the user (aka ID)
	destination: String,
	created: SystemTime,
}
