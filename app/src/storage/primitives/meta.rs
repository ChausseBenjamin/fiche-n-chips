use sqlx::FromRow;

use super::{RowKind, SystemTimeExt};
use crate::domain::{Brief, Slug};

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
