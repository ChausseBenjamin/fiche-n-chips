use sqlx::Type;

use crate::domain::RouteKind;

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
