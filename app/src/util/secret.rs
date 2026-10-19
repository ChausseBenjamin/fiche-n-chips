use std::str::FromStr;

use rslog::LogValuer;

#[derive(Clone)]
pub struct Secret<T>(T);
impl<T> Secret<T> {
	pub fn expose(&self) -> &T { &self.0 }
}

impl<T> LogValuer for Secret<T> {
	fn fmt_logged(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "REDACTED")
	}
}

impl<T> AsRef<T> for Secret<T> {
	fn as_ref(&self) -> &T { &self.0 }
}

impl<T> From<T> for Secret<T> {
	fn from(val: T) -> Self { Self(val) }
}

impl<S> FromStr for Secret<S>
where
	S: FromStr,
{
	type Err = S::Err;

	fn from_str(s: &str) -> Result<Self, Self::Err> { S::from_str(s).map(Secret) }
}
