use std::fmt::Display;
use std::path::PathBuf;

use super::{RouteKind, Storable};
use crate::reserved::ReservedPath;

pub struct Reserved(pub(super) String);

impl Storable for Reserved {
	fn id(&self) -> &str { &self.0 }

	fn kind(&self) -> RouteKind { RouteKind::Reserved }

	fn hash(&self) -> Option<i64> { None }

	fn content(&self) -> Option<&[u8]> { None }
}

impl From<&ReservedPath> for Reserved {
	fn from(value: &ReservedPath) -> Self {
		Self(value.path().display().to_string())
	}
}

impl From<PathBuf> for Reserved {
	fn from(path: PathBuf) -> Self { Self(path.to_string_lossy().to_string()) }
}

impl Display for Reserved {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.write_str(&self.0)
	}
}
