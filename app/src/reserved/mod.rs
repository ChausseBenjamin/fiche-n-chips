mod api;
mod file;
mod provider;

use std::path::{Path, PathBuf};

pub use file::{FileContents, WebFile};
pub use provider::ReservedPathProvider;

pub enum ReservedPath {
	File(WebFile<'static>),
	Api(PathBuf),
	Alerting(PathBuf),
}

impl ReservedPath {
	pub fn path(&self) -> &Path {
		match self {
			ReservedPath::File(f) => f.path(),
			ReservedPath::Api(p) | ReservedPath::Alerting(p) => p,
		}
	}
}
