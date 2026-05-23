use std::time::SystemTime;

use super::super::Error;

#[derive(Copy, Clone)]
pub enum RouteKind {
	Reserved = 0,
	Redirect = 1,
	Paste = 2,
}

impl TryFrom<i64> for RouteKind {
	type Error = Error;
	fn try_from(value: i64) -> Result<Self, Self::Error> {
		Ok(match value {
			0 => RouteKind::Reserved,
			1 => RouteKind::Redirect,
			2 => RouteKind::Paste,
			v => {
				return Err(Self::Error::ParseErr(format!(
					"{} isn't a valid route kind",
					v
				)))
			}
		})
	}
}

pub struct RouteRow {
	pub id: String,
	pub kind: RouteKind,
	pub hash: Option<i64>,
	pub content: Option<Vec<u8>>,
	pub created: Option<SystemTime>,
}

pub struct DeletedRoute {
	pub id: String,
	pub created: SystemTime,
}

pub trait Storable {
	fn id(&self) -> &str;
	fn kind(&self) -> RouteKind;
	fn hash(&self) -> Option<i64>;
	fn content(&self) -> Option<&[u8]>;
	fn created(&self) -> Option<SystemTime>;
}
