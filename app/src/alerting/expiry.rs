use std::slice::Iter;
use std::time::Duration;

use serde::Serialize;

use super::Error;
use super::emitter::{AlertMsg, MessagePriority};
use crate::storage::model::RouteKind;

#[derive(Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ContentKind {
	Paste,
	Redirect,
}

impl TryFrom<RouteKind> for ContentKind {
	type Error = Error;

	fn try_from(value: RouteKind) -> Result<Self, Self::Error> {
		use RouteKind as RK;
		match value {
			RK::Redirect => Ok(Self::Redirect),
			RK::Paste => Ok(Self::Paste),
			RK::Reserved => Err(Error::UnknownKind("reserved")),
		}
	}
}

#[derive(Serialize)]
pub struct ExpiryMsg {
	slug:    String,
	bracket: ExpiryBracket,
	kind:    ContentKind,
	pri:     MessagePriority,
}

impl Eq for ExpiryMsg {}
impl PartialEq for ExpiryMsg {
	fn eq(&self, other: &Self) -> bool {
		(self.slug == other.slug) && (self.bracket == other.bracket)
	}
}
impl Ord for ExpiryMsg {
	fn cmp(&self, other: &Self) -> std::cmp::Ordering {
		self.slug.cmp(&other.slug).then(self.bracket.cmp(&other.bracket))
	}
}
impl PartialOrd for ExpiryMsg {
	fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
		Some(self.cmp(other))
	}
}

impl AlertMsg for ExpiryMsg {
	const TEMPLATE: &'static str = "fnp-expiry";

	fn topic(&self) -> &str { &self.slug }
}

#[derive(Serialize, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum ExpiryBracket {
	#[serde(rename = "1 month")]
	Distant,
	#[serde(rename = "1 week")]
	Approaching,
	#[serde(rename = "1 day")]
	Near,
	#[serde(rename = "1 hour")]
	Imminent,
}

impl From<ExpiryBracket> for Duration {
	fn from(bracket: ExpiryBracket) -> Self {
		use ExpiryBracket as EB;
		match bracket {
			EB::Distant => Self::from_days(30),
			EB::Approaching => Self::from_weeks(1),
			EB::Near => Self::from_days(1),
			EB::Imminent => Self::from_hours(1),
		}
	}
}

impl From<&ExpiryBracket> for MessagePriority {
	fn from(bracket: &ExpiryBracket) -> Self {
		use ExpiryBracket as EB;
		match bracket {
			EB::Imminent => Self::High,
			EB::Near => Self::High,
			EB::Approaching => Self::Normal,
			EB::Distant => Self::Low,
		}
	}
}

impl ExpiryBracket {
	pub fn lead(&self) -> Duration { (*self).into() }

	pub fn iter() -> Iter<'static, Self> {
		use ExpiryBracket as EB;
		static BRACKETS: [EB; 4] =
			[EB::Distant, EB::Approaching, EB::Near, EB::Imminent];
		BRACKETS.iter()
	}

	pub fn create_warning(&self, slug: String, kind: ContentKind) -> ExpiryMsg {
		ExpiryMsg {
			slug,
			kind,
			bracket: *self,
			pri: self.into(),
		}
	}
}
