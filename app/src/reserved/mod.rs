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
			Self::File(f) => f.path(),
			Self::Api(p) | Self::Alerting(p) => p,
		}
	}
}
