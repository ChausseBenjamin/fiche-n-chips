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
}

impl ReservedPath {
	pub fn path(&self) -> &Path {
		match self {
			ReservedPath::File(f) => f.path(),
			ReservedPath::Api(p) => p,
		}
	}
}
