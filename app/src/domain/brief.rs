use std::time::SystemTime;

use super::Slug;

/// Allows describing served routes without serializing
/// data-heavy structures like [super::Entry]
pub enum RouteKind {
	Reserved,
	Redirect,
	Paste,
}

pub struct Brief {
	pub slug:    Slug,
	pub kind:    RouteKind,
	pub created: SystemTime,
}
