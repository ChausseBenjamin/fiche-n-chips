use core::fmt::Result;
use std::cmp::Ord;
use std::fmt::{Display, Formatter};
use std::ops::Sub;

/// Fuly extensible trait for logging so users can extend as they please
pub trait Level: Display {
	fn value(&self) -> i8;
}

/// Good default for the lib
/// - no user implentation needed
/// - somewhat extensible beyond 4 levels with RSlogLevel(n) without requiring a
///   custom implementation
#[derive(Eq, Ord, PartialEq, PartialOrd, Clone, Copy)]
pub struct RSlogLevel(i8);

pub const DEBUG: RSlogLevel = RSlogLevel(-4);
pub const INFO: RSlogLevel = RSlogLevel(0);
pub const WARN: RSlogLevel = RSlogLevel(4);
pub const ERROR: RSlogLevel = RSlogLevel(8);

impl Sub for RSlogLevel {
	type Output = Self;

	fn sub(self, rhs: Self) -> Self::Output { Self(self.0 - rhs.0) }
}

impl Display for RSlogLevel {
	fn fmt(&self, f: &mut Formatter<'_>) -> Result {
		match *self {
			DEBUG => write!(f, "DEBUG"),
			INFO => write!(f, "INFO"),
			WARN => write!(f, "WARN"),
			ERROR => write!(f, "ERROR"),

			_ if self > &ERROR => write!(f, "ERROR+{}", self.0 - ERROR.0),
			_ if self > &WARN => write!(f, "WARN+{}", self.0 - WARN.0),
			_ if self > &INFO => write!(f, "INFO+{}", self.0 - INFO.0),
			_ if self > &DEBUG => write!(f, "INFO{}", self.0 - INFO.0), /* neg */
			_ => write!(f, "DEBUG{}", self.0 - DEBUG.0),                /* neg */
		}
	}
}

impl From<RSlogLevel> for i8 {
	fn from(value: RSlogLevel) -> Self { value.0 }
}

impl Level for RSlogLevel {
	fn value(&self) -> i8 { self.0 }
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn sloglevel_display() {
		let inputs: Vec<RSlogLevel> = (-5..9).map(|lvl| RSlogLevel(lvl)).collect();
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

	#[test]
	fn sloglevel_values() {
		let inputs: Vec<RSlogLevel> = (-5..9).map(|lvl| RSlogLevel(lvl)).collect();

		const IS_LT_DEBUG: usize = 0;
		const IS_LT_INFO: usize = 1;
		const IS_LT_WARN: usize = 2;
		const IS_LT_ERROR: usize = 3;
		const IS_GT_ERROR: usize = 4;
		let truth_table = vec![
			// lt_debug lt_info lt_warn lt_err gt_err
			[true, true, true, true, false],     // -5
			[false, true, true, true, false],    // -4
			[false, true, true, true, false],    // -3
			[false, true, true, true, false],    // -2
			[false, true, true, true, false],    // -1
			[false, false, true, true, false],   // 00
			[false, false, true, true, false],   // 01
			[false, false, true, true, false],   // 02
			[false, false, true, true, false],   // 03
			[false, false, false, true, false],  // 04
			[false, false, false, true, false],  // 05
			[false, false, false, true, false],  // 06
			[false, false, false, true, false],  // 07
			[false, false, false, false, false], // 08
			[false, false, false, false, true],  // 09
		];

		inputs.iter().enumerate().for_each(|(i, v)| {
			let lvl = v.value();
			assert_eq!(lvl < DEBUG.value(), truth_table[i][IS_LT_DEBUG]);
			assert_eq!(lvl < INFO.value(), truth_table[i][IS_LT_INFO]);
			assert_eq!(lvl < WARN.value(), truth_table[i][IS_LT_WARN]);
			assert_eq!(lvl < ERROR.value(), truth_table[i][IS_LT_ERROR]);
			assert_eq!(lvl > ERROR.value(), truth_table[i][IS_GT_ERROR]);
		});
	}
}
