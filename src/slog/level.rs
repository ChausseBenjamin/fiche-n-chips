use core::fmt::Result;
use std::{
	cmp::Ordering,
	fmt::{Display, Formatter},
	ops::Sub,
};

/// The general trait anyone can implement if
/// - wants ready-made log levels, use the provided ones
/// - wants more granular levels without a custom implementation, wrap new u8
///   values within SlogLevel
/// - truly wants granularity, they can alway implement their own implementation
///   of the Level trait
pub trait Level<T: Display + Ord> {}

#[derive(Eq, Ord, PartialEq, PartialOrd, Clone, Copy)]
pub struct SlogLevel(i8);

pub const DEBUG: SlogLevel = SlogLevel(-4);
pub const INFO: SlogLevel = SlogLevel(0);
pub const WARN: SlogLevel = SlogLevel(4);
pub const ERROR: SlogLevel = SlogLevel(8);

impl SlogLevel {
	fn write_lvl(&self, f: &mut Formatter<'_>, base: &str) -> Result {
		match self.cmp(&SlogLevel(0)) {
			Ordering::Equal => f.write_str(base),
			Ordering::Greater => write!(f, "{}+{}", base, self.0),
			Ordering::Less => write!(f, "{}{}", base, self.0),
		}
	}
}

impl Sub for SlogLevel {
	type Output = Self;
	fn sub(self, rhs: Self) -> Self::Output {
		SlogLevel(self.0 - rhs.0)
	}
}

impl Display for SlogLevel {
	fn fmt(&self, f: &mut Formatter<'_>) -> Result {
		match self {
			&DEBUG => write!(f, "DEBUG"),
			&INFO => write!(f, "INFO"),
			&WARN => write!(f, "WARN"),
			&ERROR => write!(f, "ERROR"),

			l if self > &ERROR => write!(f, "ERROR+{}", *self - ERROR),
			l if self > &WARN => write!(f, "WARN+{}", *self - WARN),
			l if self > &INFO => write!(f, "INFO+{}", *self - INFO),
			l if self > &DEBUG => write!(f, "INFO{}", *self - INFO), /* neg */
			_ => write!(f, "DEBUG{}", *self - DEBUG),                /* neg */
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn sloglevel_display() {
		let inputs: Vec<SlogLevel> = (-5..9).map(|lvl| SlogLevel(lvl)).collect();
		let expected = vec![
			"DEBUG-1", // -5
			"DEBUG",   // -4
			"INFO-3",  // -3
			"INFO-2",  // -2
			"INFO-1",  // -1
			"INFO",    // 00
			"INFO+1",  // 01
			"INFO+2",  // 02
			"INFO+3",  // 03
			"WARN",    // 04
			"WARN+1",  // 05
			"WARN+2",  // 06
			"WARN+3",  // 07
			"ERROR",   // 08
			"ERROR+1", // 09
		];
		inputs
			.iter()
			.enumerate()
			.for_each(|(i, output)| assert_eq!(format!("{}", output), expected[i]));
	}
}
