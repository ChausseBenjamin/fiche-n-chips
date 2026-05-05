use core::fmt;

/// Rationale, if a user:
/// - wants ready-made log levels, use the provided ones
/// - wants more granular levels without a custom implementation, wrap new u8
///   values within SlogLevel
/// - truly wants granularity, they can alway implement their own implementation
///   of the Level trait
pub trait Level {
	fn value(&self) -> i8;
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result;
}

pub(crate) struct SlogLevel(i8);

pub(crate) const DEBUG: SlogLevel = SlogLevel(-4);
pub(crate) const INFO: SlogLevel = SlogLevel(0);
pub(crate) const WARN: SlogLevel = SlogLevel(4);
pub(crate) const ERROR: SlogLevel = SlogLevel(8);

impl Level for SlogLevel {
	fn value(&self) -> i8 {
		self.0
	}

	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let l = self.value();

		match l {
			l if l < INFO.value() => write_lvl(f, "DEBUG", l - DEBUG.value()),
			l if l < WARN.value() => write_lvl(f, "INFO", l - INFO.value()),
			l if l < ERROR.value() => write_lvl(f, "WARN", l - WARN.value()),
			_ => write_lvl(f, "ERROR", l - ERROR.value()),
		}
	}
}

fn write_lvl(f: &mut fmt::Formatter<'_>, base: &str, val: i8) -> fmt::Result {
	match val {
		0 => f.write_str(base),
		v if v > 0 => write!(f, "{}+{}", base, v),
		v if v < 0 => write!(f, "{}{}", base, v),
		_ => unreachable!(),
	}
}
