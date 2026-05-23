use std::time::SystemTime;

use super::{RouteKind, Storable};

pub struct ReservedPath(pub(super) String);

impl Storable for ReservedPath {
	fn id(&self) -> &str {
		&self.0
	}
	fn kind(&self) -> RouteKind {
		RouteKind::Reserved
	}
	fn hash(&self) -> Option<i64> {
		None
	}
	fn content(&self) -> Option<&[u8]> {
		None
	}
	fn created(&self) -> Option<SystemTime> {
		None
	}
}
