mod api;
mod file;
mod provider;

use std::path::Path;

pub use api::HEALTHCHECK_PATH;
pub use file::{FileContents, WebFile};
pub use provider::ReservedPathProvider;

pub enum ReservedPath {
	File(WebFile<'static>),
	Api(&'static Path),
	Alerting(&'static Path),
}

impl ReservedPath {
	pub fn path(&self) -> &Path {
		match self {
			Self::File(f) => f.path(),
			Self::Api(p) | Self::Alerting(p) => p,
		}
	}
}
