use std::borrow::Cow;
use std::str::FromStr;

use super::{Error, Result};
use crate::util::normalize_http;

const FORBIDDEN_CHARS: [char; 9] =
	['.', '?', '&', '/', '#', '\\', '"', '%', '^'];

pub struct Slug(Cow<'_, str>);
impl<T: FromStr> TryFrom<T> for Slug {
	type Error = Error;

	fn try_from(s: T) -> Result<Self> {
		if let Some(c) = s.chars().find(|c| FORBIDDEN_CHARS.contains(c)) {
			return Err(super::Error::Parse(format!(
				"slug contains undesirable character: '{c}'"
			)));
		}
		let normalized = normalize_http(Path::new(s)).to_string_lossy();
		if normalized.len() != s.len() {
			return Err(Error::Parse(format!(
				"slug would be mangled by path normalization (`{s}` -> `{normalized}`)"
			)));
		}
		Ok(Self(normalized))
	}
}
