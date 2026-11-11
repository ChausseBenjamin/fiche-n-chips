use std::fmt::{self, Display, Formatter};
use std::path::Path;
use std::str::FromStr;

use super::{Error, Result};
use crate::util::normalize_http;

const FORBIDDEN_CHARS: [char; 9] =
	['.', '?', '&', '/', '#', '\\', '"', '%', '^'];

pub struct Slug(Box<str>);
impl Slug {
	pub(crate) fn from_validated(raw: Box<str>) -> Self { Self(raw) }

	pub fn as_str(&self) -> &str { &self.0 }

	pub fn parse(raw: impl Into<Box<str>>) -> Result<Self> {
		let s = raw.into();
		if let Some(c) = s.chars().find(|c| FORBIDDEN_CHARS.contains(c)) {
			return Err(super::Error::Parse(format!(
				"slug contains undesirable character: '{c}'"
			)));
		}
		let normalized = normalize_http(Path::new(&*s));
		let normalized = normalized.to_string_lossy();
		if normalized.len() != s.len() {
			return Err(Error::Parse(format!(
				"slug would be mangled by path normalization (`{s}` -> `{}`)",
				&normalized
			)));
		}
		Ok(Self(s))
	}
}

impl Display for Slug {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result { f.write_str(&self.0) }
}
