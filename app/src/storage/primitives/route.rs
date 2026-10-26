use std::time::SystemTime;

use bytes::Bytes;
use sqlx::{FromRow, Type};

use super::super::{Error, Result};
use super::SystemTimeExt;
use crate::domain::{
	Blob, Brief, Brotli, Checksum, Content, Entry, Existing, Record, RouteKind,
	Slug,
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
	id:      Box<str>,
	kind:    RowKind,
	hash:    Option<i64>,
	content: Option<Box<[u8]>>,
	#[sqlx(rename = "created_on", try_from = "SystemTimeExt")]
	created: SystemTime,
}
impl TryFrom<Route> for Record<Brotli, Existing> {
	type Error = Error;

	fn try_from(r: Route) -> Result<Self> {
		use RowKind as RK;
		let (slug, created) = (Slug::from_validated(r.id), Existing(r.created));
		let entry = match (r.kind, r.content, r.hash) {
			(RK::Reserved, _, _) => Entry::Reserved,
			(RK::Redirect, Some(content), _) => {
				Entry::Served(Content::Redirect(Brotli(Bytes::from(content))))
			}
			(RK::Paste, Some(content), Some(hash)) => {
				Entry::Served(Content::Paste(Blob::<Brotli>::from_compressed(
					Bytes::from(content),
					Checksum::from(hash),
				)))
			}
			(RK::Paste, Some(_), None) => {
				return Err(Error::Formatting(format!(
					"paste route `{slug}` has no associated hash",
				)));
			}
			(RK::Redirect, None, _) | (RK::Paste, None, _) => {
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
	id:      Box<str>,
	kind:    RowKind,
	#[sqlx(rename = "created_on", try_from = "SystemTimeExt")]
	created: SystemTime,
}
impl From<Meta> for Brief {
	fn from(meta: Meta) -> Self {
		Self {
			slug:    Slug::from_validated(meta.id),
			kind:    meta.kind.into(),
			created: meta.created,
		}
	}
}
