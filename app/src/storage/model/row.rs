use std::time::SystemTime;

use super::paste::CompressedPaste;
use super::redirect::Redirect;
use super::{Error, Reserved, Result};
use crate::storage::model::redirect::CompressedRedirect;

#[derive(Copy, Clone)]
pub enum RouteKind {
	Reserved = 0,
	Redirect = 1,
	Paste    = 2,
}

impl TryFrom<i64> for RouteKind {
	type Error = Error;

	fn try_from(value: i64) -> Result<Self> {
		Ok(match value {
			0 => RouteKind::Reserved,
			1 => RouteKind::Redirect,
			2 => RouteKind::Paste,
			v => {
				return Err(Self::Error::ParseErr(format!(
					"{v} isn't a valid route kind"
				)))
			}
		})
	}
}

pub struct RouteRow {
	pub id:      String,
	pub kind:    RouteKind,
	pub hash:    Option<i64>,
	pub content: Option<Vec<u8>>,
	pub created: Option<SystemTime>,
}

pub struct DeletedRoute {
	pub id:      String,
	pub created: SystemTime,
}

pub trait Storable {
	fn id(&self) -> &str;
	fn kind(&self) -> RouteKind;
	fn hash(&self) -> Option<i64>;
	fn content(&self) -> Option<&[u8]>;
}

pub enum RowEntry {
	Reserved(Reserved),
	Redirect(Redirect, SystemTime),
	Paste(CompressedPaste, SystemTime),
}

impl TryFrom<RouteRow> for RowEntry {
	type Error = super::Error;

	fn try_from(value: RouteRow) -> Result<Self> {
		use RowEntry as RE;
		Ok(match value.kind {
			RouteKind::Reserved => RE::Reserved(Reserved(value.id)),
			RouteKind::Redirect => {
				RE::Redirect(
					Redirect::try_from(CompressedRedirect {
						short:       value.id,
						// TODO: Err for missing content (add case to enum?)
						destination: value.content.unwrap_or_default(),
					})?,
					value.created.ok_or_else(|| {
						Error::ParseErr("missing created_on for redirect".into())
					})?,
				)
			}
			RouteKind::Paste => RE::Paste(
				CompressedPaste {
					slug:    value.id,
					// TODO: Err for missing hash and/or content (same as above)
					hash:    value.hash.unwrap_or_default().cast_unsigned(),
					content: value.content.unwrap_or_default(),
				},
				value.created.ok_or_else(|| {
					Error::ParseErr("missing created_on for paste".into())
				})?,
			),
		})
	}
}
