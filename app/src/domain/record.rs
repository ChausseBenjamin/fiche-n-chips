use std::borrow::Cow;
use std::time::SystemTime;

use bytes::Bytes;

use super::{Brotli, Checksum, Plain, Result, RouteKind, Slug};

pub type RecordPacked = Record<Brotli>;
pub type RecordPlain = Record<Plain>;

pub struct Record<E> {
	pub slug:    Slug,
	pub entry:   Entry<E>,
	pub created: SystemTime,
}

pub type EntryPacked = Entry<Brotli>;
pub type EntryPlain = Entry<Plain>;

pub enum Entry<E> {
	Reserved, // holds no contents, just blocks paths/slugs
	Served(Content<E>),
}

pub type ContentPlain = Content<Plain>;
pub type ContentPacked = Content<Brotli>;
pub enum Content<E> {
	Paste(E),
	Redirect(E),
}
impl<E: AsRef<[u8]>> Content<E> {
	pub fn len(&self) -> usize {
		let bytes = match self {
			Self::Paste(v) | Self::Redirect(v) => v.as_ref(),
		};
		bytes.len()
	}
}
impl ContentPlain {
	pub fn redirect(bytes: impl Into<Box<[u8]>>) -> Self {
		Self::Redirect(Plain(bytes.into()))
	}

	pub fn paste(bytes: impl Into<Box<[u8]>>) -> Self {
		Self::Paste(Plain(bytes.into()))
	}

	pub(super) fn compress(self) -> Result<ContentPacked> {
		use Content::{Paste, Redirect};
		Ok(match self {
			Self::Paste(plain) => Paste(plain.compress()?),
			Self::Redirect(plain) => Redirect(plain.compress()?),
		})
	}

	pub(super) fn bytes(&self) -> &[u8] {
		match self {
			Self::Paste(v) | Self::Redirect(v) => &v.0,
		}
	}
}
impl ContentPacked {
	pub fn from_stored_paste(bytes: Box<[u8]>) -> Self {
		Self::Paste(Brotli(bytes))
	}

	pub fn from_stored_redir(bytes: Box<[u8]>) -> Self {
		Self::Redirect(Brotli(bytes))
	}

	pub fn into_inner(self) -> Box<[u8]> {
		match self {
			Self::Paste(v) | Self::Redirect(v) => v.0,
		}
	}
}
