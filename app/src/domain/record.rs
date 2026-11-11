use std::borrow::Cow;
use std::time::SystemTime;

use super::{AsPlainRef, Brotli, Checksum, Plain, Result, RouteKind, Slug};

pub type StoredRecord = Record<Brotli, Existing>;

pub struct New;
pub struct Existing(pub SystemTime);
pub struct Record<E, S> {
	pub slug:    Slug,
	pub entry:   Entry<E>,
	pub created: S, // Either New or Existing (in storage already)
}

pub enum Entry<E> {
	Reserved, // holds no contents, just blocks paths/slugs
	Served(Content<E>),
}

pub enum Content<E> {
	Paste(E),
	Redirect(E),
}
impl<E: AsPlainRef> Content<E> {
	pub fn raw_ref(&self) -> Result<Cow<'_, [u8]>> {
		match self {
			Self::Paste(blob) => blob.as_plain_ref(),
			Self::Redirect(redir) => redir.as_plain_ref(),
		}
	}
}
