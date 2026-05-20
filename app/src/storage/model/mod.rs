use std::time::SystemTime;

pub mod paste;
pub mod redirect;
pub mod reserved;

use paste::*;
use redirect::*;
use reserved::*;

/// Meant to receive data from the DB or prep them for insertion.
/// (Agnostic because it doesn't know what kind it is)
pub struct AgnosticRoute {
	pub id: String,
	pub hash: Option<i64>,
	pub kind: RouteKind,
	pub content: Option<Vec<u8>>, // Raw-bytes (aka Blob)
	pub created: SystemTime,
}

#[derive(Copy, Clone)]
pub enum RouteKind {
	Reserved = 0,
	Redirect = 1,
	Paste = 2,
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
				                            * and xxh3 */
				kind: RouteKind::Paste,
				content: Some(p.content),
				created: p.created,
			},
		}
	}
}
