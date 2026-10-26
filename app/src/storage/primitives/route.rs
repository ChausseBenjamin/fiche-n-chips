use std::time::SystemTime;

use bytes::Bytes;
use sqlx::{FromRow, Type};

use super::super::{Error, Result};
use super::SystemTimeExt;
use crate::domain::{
	Brief, Brotli, Checksum, Content, ContentPacked, Entry, Novel, Record,
	RecordStored, RouteKind, Slug,
};

/// Decouples storage implementation from the [RouteKind] domain object
#[derive(Debug, Eq, PartialEq, Ord, PartialOrd, Type)]
#[repr(u8)]
pub enum RowKind {
	Reserved = 0,
	Redirect = 1,
	Paste    = 2,
}
impl From<RowKind> for RouteKind {
	fn from(kind: RowKind) -> Self {
		use RowKind as RK;
		match kind {
			RK::Reserved => Self::Reserved,
			RK::Redirect => Self::Redirect,
			RK::Paste => Self::Paste,
		}
	}
}

#[derive(FromRow)]
pub struct Route {
	pub id:      Box<str>,
	pub kind:    RowKind,
	pub content: Option<Box<[u8]>>,
	pub created: SystemTimeExt,
}
impl TryFrom<Route> for RecordStored {
	type Error = Error;

	fn try_from(r: Route) -> Result<Self> {
		use RowKind as RK;
		let slug = Slug::from_validated(r.id);
		let created = Novel(r.created.into());
		let entry = match (r.kind, r.content) {
			(RK::Reserved, _) => Entry::Reserved,
			(RK::Redirect, Some(data)) => {
				Entry::Served(ContentPacked::from_stored_redir(Bytes::from(data)))
			}
			(RK::Paste, Some(data)) => {
				Entry::Served(ContentPacked::from_stored_paste(Bytes::from(data)))
			}
			(RK::Redirect, None) | (RK::Paste, None) => {
				return Err(Error::Formatting(format!(
					"non-reserved route `{slug}` has no content",
				)));
			}
		};
		Ok(Self {
			slug,
			entry,
			created,
		})
	}
}

#[derive(FromRow)]
pub struct Meta {
	pub id:      Box<str>,
	pub kind:    RowKind,
	pub created: SystemTimeExt,
}
impl From<Meta> for Brief {
	fn from(meta: Meta) -> Self {
		Self {
			slug:    Slug::from_validated(meta.id),
			kind:    meta.kind.into(),
			created: meta.created.into(),
		}
	}
}
