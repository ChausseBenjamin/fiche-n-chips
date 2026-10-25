use std::borrow::Cow;
use std::time::SystemTime;

use super::{AsPlainRef, Blob, Brotli, Plain, Result, Slug};
use crate::storage::RouteKind;

pub enum Entry<E> {
	Reserved, // holds no contents, just blocks paths/slugs
	Served(Content<E>),
}

pub enum Content<E> {
	Paste(Blob<E>),
	Redirect(E),
}
impl<E: AsPlainRef> Content<E> {
	pub fn raw_ref(&self) -> Result<Cow<'_, [u8]>> {
		match self {
			Self::Paste(blob) => blob.bytes.as_plain_ref(),
			Self::Redirect(redir) => redir.as_plain_ref(),
		}
	}
}

pub(super) struct New;
pub(super) struct Existing(pub SystemTime);
pub struct Record<S, E> {
	pub(super) slug:    Slug,
	pub(super) entry:   Content<E>,
	pub(super) created: S, // Either New or Existing (in storage already)
}

pub struct Meta {
	pub slug:    Slug,
	pub created: SystemTime,
}
