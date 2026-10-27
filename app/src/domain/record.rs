use std::borrow::Cow;
use std::time::SystemTime;

use bytes::Bytes;

use super::{AsPlainRef, Brotli, Checksum, Plain, Result, RouteKind, Slug};

pub type RecordPending = Record<Brotli, Novel>;
pub type RecordStored = Record<Brotli, Existing>;

pub struct Novel(Option<Checksum>);
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

pub type ContentPlain = Content<Plain>;
pub type ContentPacked = Content<Brotli>;
pub(super) enum Content<E> {
	Paste(E),
	Redirect(E),
}
impl ContentPlain {
	pub fn redirect(bytes: Vec<u8>) -> Self { Self::Redirect(Plain(bytes)) }

	pub fn paste(bytes: Vec<u8>) -> Self { Self::Paste(Plain(bytes)) }

	pub(super) fn compress(self) -> Result<ContentPacked> {
		use Content::{Paste, Redirect};
		Ok(match self {
			Self::Paste(plain) => Paste(plain.compress()?),
			Self::Redirect(plain) => Redirect(plain.compress()?),
		})
	}

	pub(super) fn bytes(&self) -> Vec<u8> {
		match self {
			Self::Paste(v) | Self::Redirect(v) => v.0,
		}
	}
}
impl ContentPacked {
	pub fn from_stored_paste(bytes: Vec<u8>) -> Self { Self(Brotli(bytes)) }

	pub fn from_stored_redir(bytes: Vec<u8>) -> Self { Self(Brotli(bytes)) }

	pub fn into_inner(self) -> Vec<u8> {
		match self {
			Self::Paste(v) | Self::Redirect(v) => v.0,
		}
	}
}
