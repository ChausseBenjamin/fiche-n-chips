use core::fmt::Result;
use std::{
	cmp::{Ord, Ordering},
	fmt::{Display, Formatter},
	ops::Sub,
};

/// Fuly extensible trait for logging so users can extend as they please
pub trait Level {
	fn disp(&self, f: &mut Formatter<'_>) -> Result;
	fn value(&self) -> i8;
}

/// Good default for the lib
/// - no user implentation needed
/// - somewhat extensible beyon 4 levels with SlogLevel(n) without requiring a
///   custom implementation
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

			l if self > &ERROR => write!(f, "ERROR+{}", self.0 - ERROR.0),
			l if self > &WARN => write!(f, "WARN+{}", self.0 - WARN.0),
			l if self > &INFO => write!(f, "INFO+{}", self.0 - INFO.0),
			l if self > &DEBUG => write!(f, "INFO{}", self.0 - INFO.0), /* neg */
			_ => write!(f, "DEBUG{}", self.0 - DEBUG.0),                /* neg */
		}
	}
}

impl Level for SlogLevel {
	fn disp(&self, f: &mut Formatter<'_>) -> Result {
		self.fmt(f)
	}

	fn value(&self) -> i8 {
		self.0
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

	#[test]
	fn sloglevel_values() {
		let inputs: Vec<SlogLevel> = (-5..9).map(|lvl| SlogLevel(lvl)).collect();

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
