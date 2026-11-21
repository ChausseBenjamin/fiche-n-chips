use std::time::SystemTime;

use super::Slug;
use super::record::Content;

/// Allows describing served routes without serializing
/// data-heavy structures like [super::Entry]
#[derive(Copy, Clone)]
pub enum RouteKind {
	Reserved,
	Redirect,
	Paste,
}
impl<E> From<&Content<E>> for RouteKind {
	fn from(value: &Content<E>) -> Self {
		match value {
			Content::Paste(_) => Self::Paste,
			Content::Redirect(_) => Self::Redirect,
		}
	}
}

pub struct Brief {
	pub slug:    Slug,
	pub kind:    RouteKind,
	pub created: SystemTime,
}
