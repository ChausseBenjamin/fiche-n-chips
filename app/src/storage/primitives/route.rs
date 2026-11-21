use sqlx::FromRow;

use super::super::{Error, Result};
use super::{RowKind, SystemTimeExt};
use crate::domain::{ContentPacked, EntryPacked, RecordPacked, Slug};

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
