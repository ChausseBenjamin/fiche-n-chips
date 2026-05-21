use std::time::SystemTime;

mod paste;
mod redirect;
mod reserved;

use paste::CompPaste;
pub use paste::{HashedPaste, MatchingPaste};
use redirect::*;
use reserved::*;

/// Meant to receive data from the DB or prep them for insertion.
/// (Agnostic because it doesn't know what kind it is)
pub struct AgnosticRoute {
	pub id: String,
	pub kind: RouteKind,
	pub hash: Option<i64>,
	pub content: Option<Vec<u8>>, // Raw-bytes (aka Blob)
	pub created: SystemTime,
}

pub struct DeletedRoute {
	pub id: String,
	pub created: SystemTime,
}

#[derive(Copy, Clone)]
pub enum RouteKind {
	Reserved = 0,
	Redirect = 1,
	Paste = 2,
}

impl TryFrom<i64> for RouteKind {
	type Error = super::Error;
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

pub enum Route {
	Reserved(Reserved),
	Redirect(CompRedirect),
	Paste(CompPaste),
}

impl From<Route> for AgnosticRoute {
	fn from(value: Route) -> Self {
		match value {
			Route::Reserved(r) => AgnosticRoute {
				id: r.0,
				hash: None,
				kind: RouteKind::Reserved,
				content: None,
				created: SystemTime::MIN, // Date doesn't matter here...
			},
			Route::Redirect(r) => AgnosticRoute {
				id: r.short,
				hash: None,
				kind: RouteKind::Redirect,
				content: Some(r.destination),
				created: r.created,
			},
			Route::Paste(p) => AgnosticRoute {
				id: p.slug,
				hash: Some(p.hash as i64), /* bridges between sqlite INTEGER (i64)
				                            * and xxh3 (u64) */
				kind: RouteKind::Paste,
				content: Some(p.content),
				created: p.created,
			},
		}
	}
}
