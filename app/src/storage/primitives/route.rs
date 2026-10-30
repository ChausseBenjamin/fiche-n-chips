use sqlx::{FromRow, Type};

use super::super::{Error, Result};
use super::SystemTimeExt;
use crate::domain::{
	Brief, Brotli, ContentPacked, EntryPacked, RecordPacked, RouteKind, Slug,
};

#[derive(FromRow)]
pub struct Payload {
	pub kind:    RowKind,
	pub content: Option<Box<[u8]>>,
}
impl TryFrom<Payload> for EntryPacked {
	type Error = Error;

	fn try_from(p: Payload) -> Result<EntryPacked> {
		use RowKind as RK;
		Ok(match (p.kind, p.content) {
			(RK::Reserved, _) => EntryPacked::Reserved,
			(RK::Redirect, Some(data)) => {
				EntryPacked::Served(ContentPacked::from_stored_redir(data))
			}
			(RK::Paste, Some(data)) => {
				EntryPacked::Served(ContentPacked::from_stored_paste(data))
			}
			(kind, None) => {
				return Err(Error::Formatting(format!(
					"non-reserved route of kind {kind:?} has no content",
				)));
			}
		})
	}
}

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
impl From<RouteKind> for RowKind {
	fn from(kind: RouteKind) -> Self {
		use RouteKind as DK;
		match kind {
			DK::Reserved => Self::Reserved,
			DK::Redirect => Self::Redirect,
			DK::Paste => Self::Paste,
		}
	}
}
impl From<RowKind> for i64 {
	fn from(kind: RowKind) -> Self { kind as Self }
}

#[derive(FromRow)]
pub struct Route {
	pub id:      Box<str>,
	pub kind:    RowKind,
	pub content: Option<Box<[u8]>>,
	pub created: SystemTimeExt,
}
impl TryFrom<Route> for RecordPacked {
	type Error = Error;

	fn try_from(r: Route) -> Result<Self> {
		use RowKind as RK;
		let slug = Slug::from_validated(r.id);
		let created = r.created.into();
		let payload = Payload {
			kind:    r.kind,
			content: r.content,
		};
		let entry = EntryPacked::try_from(payload)?;
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
