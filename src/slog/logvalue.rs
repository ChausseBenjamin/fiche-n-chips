use std::fmt::{Display, Formatter, Result};

/// Special trait anyone can implement so values are treated differently when
/// logged vs displayed.
pub trait LogValuer {
	fn fmt_logged(&self, f: &mut Formatter<'_>) -> Result;
}

#[derive(Debug, PartialEq)]
pub enum LogValue<D: Display, L: LogValuer> {
	Vanilla(D),
	LogFmt(L),
}

impl<D: Display, L: LogValuer> Display for LogValue<D, L> {
	fn fmt(&self, f: &mut Formatter<'_>) -> Result {
		match self {
			Self::Vanilla(v) => v.fmt(f),
			Self::LogFmt(l) => l.fmt_logged(f),
		}
	}
}

#[doc(hidden)]
pub mod __prefer_log_fmt {
	use std::fmt::Display;

	use super::{LogValue, LogValuer};

	#[derive(Debug, PartialEq)]
	pub enum Never {}
	impl Display for Never {
		fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
			match *self {}
		}
	}
	impl LogValuer for Never {
		fn fmt_logged(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
			match *self {}
		}
	}

	pub struct Wrap<T>(pub T);

	// Inherent impl -> wins over trait impl when T: LogValuer.
	// Returns FooBar<Never, T>: The Display slot is dead.
	impl<T: LogValuer> Wrap<T> {
		pub fn resolve(self) -> LogValue<Never, T> {
			LogValue::LogFmt(self.0)
		}
	}

	// Fallback trait -> only reached when T: !LogValuer.
	// Returns LogValue<T, Never>: The LogFmt slot is dead.
	pub trait ResolveFallback {
		type Inner: Display;
		fn resolve(self) -> LogValue<Self::Inner, Never>;
	}

	impl<T: Display> ResolveFallback for Wrap<T> {
		type Inner = T;
		fn resolve(self) -> LogValue<T, Never> {
			LogValue::Vanilla(self.0)
		}
	}
}

#[macro_export]
macro_rules! log_val {
	($val:expr) => {{
		// Bring the fallback trait into scope without polluting the caller's
		// namespace. Inherent methods shadow this automatically when T: LogValuer.
		use $crate::logvalue::__prefer_log_fmt::ResolveFallback as _;
		$crate::logvalue::__prefer_log_fmt::Wrap($val).resolve()
	}};
}

#[cfg(test)]
mod tests {
	use std::fmt::Display;

	use super::*;

	const DISP_TMPL: &str = "When displayed: ";
	const LOG_TMPL: &str = "When logged: ";

	#[derive(Debug, PartialEq, Clone)]
	pub struct OnlyDisplay;
	impl Display for OnlyDisplay {
		fn fmt(&self, f: &mut Formatter<'_>) -> Result {
			f.write_str("regular value")
		}
	}

	/// Type that doesn't implement the Display trait but the user wants
	/// a custom log format for it.
	#[derive(Debug, PartialEq, Clone)]
	pub struct OnlyLogfmt;
	impl LogValuer for OnlyLogfmt {
		fn fmt_logged(&self, f: &mut Formatter<'_>) -> Result {
			f.write_str("special logging format")
		}
	}

	/// Type that does implement Display, but must be handled differently in logs:
	/// Example: secrets
	/// a custom log format for it.
	#[derive(Debug, PartialEq, Clone)]
	pub struct DisplayAndLog;
	impl Display for DisplayAndLog {
		fn fmt(&self, f: &mut Formatter<'_>) -> Result {
			f.write_str("Pa$$w0rd")
		}
	}
	impl LogValuer for DisplayAndLog {
		fn fmt_logged(&self, f: &mut Formatter<'_>) -> Result {
			f.write_str("REDACTED")
		}
	}

	#[test]
	fn only_has_display() {
		let input = OnlyDisplay;
		let input_wrapped = log_val!(input.clone());

		let expected_wrap = LogValue::Vanilla(OnlyDisplay);
		let expected_disp = "When displayed: `regular value`";
		let expected_log = "When logged: `regular value`";

		assert_eq!(input_wrapped, expected_wrap);
		assert_eq!(format!("{}`{}`", DISP_TMPL, input), expected_disp);
		assert_eq!(format!("{}`{}`", LOG_TMPL, input_wrapped), expected_log);
	}

	#[test]
	fn only_has_logfmt() {
		let input = OnlyLogfmt;
		let input_wrapped = log_val!(input.clone());

		let expected_wrap = LogValue::LogFmt(OnlyLogfmt);
		// Cannot display: Not tested
		let expected_log = "When logged: `special logging format`";

		assert_eq!(input_wrapped, expected_wrap);
		assert_eq!(format!("{}`{}`", LOG_TMPL, input_wrapped), expected_log);
	}

	#[test]
	fn logfmt_priority_when_both() {
		let input = DisplayAndLog;
		let input_wrapped = log_val!(input.clone());

		let expected_wrap = LogValue::LogFmt(DisplayAndLog);
		let expected_disp = "When displayed: `Pa$$w0rd`";
		let expected_log = "When logged: `REDACTED`";

		assert_eq!(input_wrapped, expected_wrap);
		assert_eq!(format!("{}`{}`", DISP_TMPL, input), expected_disp);
		assert_eq!(format!("{}`{}`", LOG_TMPL, input_wrapped), expected_log);
	}
}
