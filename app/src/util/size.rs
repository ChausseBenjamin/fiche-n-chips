use std::fmt;
use std::str::FromStr;

use rslog::LogValuer;

const KILO: u64 = 1000;
const MEGA: u64 = 1000u64.pow(2);
const GIGA: u64 = 1000u64.pow(3);
const TERA: u64 = 1000u64.pow(4);

const KIBI: u64 = 1024;
const MEBI: u64 = 1024u64.pow(2);
const GIBI: u64 = 1024u64.pow(3);
const TEBI: u64 = 1024u64.pow(4);

const BINARY: &[(u64, &str)] =
	&[(TEBI, "TiB"), (GIBI, "GiB"), (MEBI, "MiB"), (KIBI, "KiB")];
const DECIMAL: &[(u64, &str)] =
	&[(TERA, "TB"), (GIGA, "GB"), (MEGA, "MB"), (KILO, "kB")];

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct StorageSize(pub u64);

impl StorageSize {
	pub fn get(self) -> u64 { self.0 }
}

impl fmt::Display for StorageSize {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "{}", self.0)
	}
}

impl LogValuer for StorageSize {
	#[allow(clippy::cast_precision_loss)]
	fn fmt_logged(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let v = self.0;

		for &(unit, suffix) in BINARY {
			if v >= unit && v.is_multiple_of(unit) {
				return write!(f, "{} {suffix}", v / unit);
			}
		}
		for &(unit, suffix) in DECIMAL {
			if v >= unit && v.is_multiple_of(unit) {
				return write!(f, "{} {suffix}", v / unit);
			}
		}
		for &(unit, suffix) in BINARY {
			if v >= unit {
				return write!(f, "{:.2} {suffix}", v as f64 / unit as f64);
			}
		}

		write!(f, "{v} B")
	}
}

enum Unit {
	Binary(u64),
	Decimal(u64),
}

fn resolve(first: u8, second: Option<u8>) -> Result<Unit, String> {
	let second = second.map(|b| b as char);

	if let Some(c) = second {
		if !matches!(c, 'B' | 'b' | 'i') {
			return Err("unknown suffix modifier".to_string());
		}
	}

	let use_binary = first.is_ascii_uppercase() || second == Some('i');

	let (decimal, binary) = match first.to_ascii_lowercase() {
		b'k' => (KILO, KIBI),
		b'm' => (MEGA, MEBI),
		b'g' => (GIGA, GIBI),
		b't' => (TERA, TEBI),
		_ => return Err("unknown unit prefix".to_string()),
	};

	if use_binary {
		Ok(Unit::Binary(binary))
	} else {
		Ok(Unit::Decimal(decimal))
	}
}

#[allow(clippy::cast_precision_loss)]
fn parse_size(s: &str) -> Result<f64, String> {
	let s = s.trim();
	if s.is_empty() {
		return Err("empty string".into());
	}

	let suffix_start = s
		.find(|c: char| !c.is_ascii_digit() && c != '.' && c != ' ')
		.unwrap_or(s.len());

	let (digits, suffix) = s.split_at(suffix_start);
	let suffix = suffix.trim();
	let value: f64 = digits
		.trim()
		.parse()
		.map_err(|_| format!("invalid storage size: {s}"))?;

	if suffix.is_empty() {
		return Ok(value);
	}

	let raw = suffix.as_bytes();

	let multiplier = if raw.len() == 1 {
		match raw[0] as char {
			'B' => 1.0,
			'b' => 1.0 / 8.0,
			'K' => KIBI as f64,
			'M' => MEBI as f64,
			'G' => GIBI as f64,
			'T' => TEBI as f64,
			_ => return Err(format!("unknown suffix: {suffix}")),
		}
	} else if raw.len() == 2 {
		let n = match resolve(raw[0], Some(raw[1]))? {
			Unit::Decimal(n) | Unit::Binary(n) => n,
		};
		if raw[1] == b'b' {
			n as f64 / 8.0
		} else {
			n as f64
		}
	} else if raw.len() == 3 {
		if raw[1] as char != 'i' {
			return Err(format!("unknown suffix: {suffix}"));
		}
		let unit = resolve(raw[0], Some(b'i'))?;
		match (unit, raw[2] as char) {
			(Unit::Binary(n), 'B') => n as f64,
			(Unit::Binary(n), 'b') => n as f64 / 8.0,
			_ => return Err(format!("unknown suffix: {suffix}")),
		}
	} else {
		return Err(format!("unknown suffix: {suffix}"));
	};

	Ok(value * multiplier)
}

impl FromStr for StorageSize {
	type Err = String;

	#[allow(
		clippy::cast_precision_loss,
		clippy::cast_possible_truncation,
		clippy::cast_sign_loss
	)]
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		let v = parse_size(s)?;
		if !(0.0..=u64::MAX as f64).contains(&v) {
			return Err("storage size out of range".into());
		}
		Ok(StorageSize(v.round() as u64))
	}
}

#[cfg(test)]
mod tests {
	use std::str::FromStr;

	use super::StorageSize as SSize;

	#[test]
	fn parsing() {
		#[rustfmt::skip]
		[
			("0",         SSize(0),                 "raw byte: zero"),
			("1",         SSize(1),                 "raw byte: one"),
			("1024",      SSize(1024),              "raw byte: 1024"),
			("1048576",   SSize(1048576),           "raw byte: 1 MiB"),
			("0B",        SSize(0),                 "B suffix: zero"),
			("256B",      SSize(256),               "B suffix: 256"),
			("8b",        SSize(1),                 "b suffix: 8 bits = 1 byte"),
			("1b",        SSize(0),                 "b suffix: 1 bit → 0 bytes (truncated)"),
			("1024b",     SSize(128),               "b suffix: 1024 bits = 128 bytes"),
			("1kB",       SSize(1000),              "decimal kB: 1 kilobyte = 1000"),
			("2kB",       SSize(2000),              "decimal kB: 2 kilobytes = 2000"),
			("1.5kB",     SSize(1500),              "decimal kB: fractional"),
			("1KB",       SSize(1024),              "binary KB: 1 kilobyte = 1024"),
			("2KB",       SSize(2048),              "binary KB: 2 kilobytes = 2048"),
			("1kiB",      SSize(1024),              "kiB (lower k): 1 kibibyte"),
			("1KiB",      SSize(1024),              "KiB: 1 kibibyte"),
			("2kiB",      SSize(2048),              "kiB: 2 kibibytes"),
			("1.5kiB",    SSize(1536),              "kiB: fractional"),
			("1K",        SSize(1024),              "K (single uppercase): binary shorthand"),
			("2K",        SSize(2048),              "K: 2 KiB"),
			("1M",        SSize(1048576),           "M: 1 MiB shorthand"),
			("1G",        SSize(1073741824),        "G: 1 GiB shorthand"),
			("1T",        SSize(1099511627776),     "T: 1 TiB shorthand"),
			("1mB",       SSize(1_000_000),         "decimal mB: 1 megabyte"),
			("2mB",       SSize(2_000_000),         "decimal mB: 2 megabytes"),
			("1MiB",      SSize(1_048_576),         "MiB: 1 mebibyte"),
			("2MiB",      SSize(2_097_152),         "MiB: 2 mebibytes"),
			("1MB",       SSize(1_048_576),         "MB (upper): binary mebibyte"),
			("1gB",       SSize(1_000_000_000),     "decimal gB: 1 gigabyte"),
			("2gB",       SSize(2_000_000_000),     "decimal gB: 2 gigabytes"),
			("1GiB",      SSize(1_073_741_824),     "GiB: 1 gibibyte"),
			("2GiB",      SSize(2_147_483_648),     "GiB: 2 gibibytes"),
			("1tB",       SSize(1_000_000_000_000), "decimal tB: 1 terabyte"),
			("1TiB",      SSize(1_099_511_627_776), "TiB: 1 tebibyte"),
			("1kb",       SSize(125),               "decimal kb: 1 kilobit = 125 bytes"),
			("8kb",       SSize(1000),              "decimal kb: 8 kilobits = 1 kB"),
			("1Kb",       SSize(128),               "binary Kb: 1 kilobit = 128 bytes"),
			("8Kb",       SSize(1024),              "binary Kb: 8 kilobits = 1 KiB"),
			("1kib",      SSize(128),               "kib: 1 kibibit = 128 bytes"),
			("8kib",      SSize(1024),              "kib: 8 kibibits = 1 KiB"),
			("1Kib",      SSize(128),               "Kib: 1 kibibit = 128 bytes"),
			("1mb",       SSize(125_000),           "decimal mb: 1 megabit"),
			("8mb",       SSize(1_000_000),         "decimal mb: 8 megabits = 1 MB"),
			("1Mib",      SSize(131_072),           "Mib: 1 mebibit"),
			("8Mib",      SSize(1_048_576),         "Mib: 8 mebibits = 1 MiB"),
			("  256KB  ", SSize(262_144),           "trailing/leading spaces stripped"),
			("0.5kB",     SSize(500),               "fractional decimal kB"),
			("0.5kiB",    SSize(512),               "fractional kiB"),
			("2.5mB",     SSize(2_500_000),         "fractional decimal mB"),
		]
		.into_iter()
		.for_each(|(input, expected, reason)| {
			assert_eq!(SSize::from_str(input).unwrap(), expected, "{reason}");
		});
	}

	fn format_logged(s: SSize) -> String { format!("{}", rslog::log_val!(s)) }

	#[test]
	fn display_shows_raw_bytes() {
		[
			(SSize(1024), "1024", "Display for 1024"),
			(SSize(1_048_576), "1048576", "Display for 1 MiB"),
		]
		.into_iter()
		.for_each(|(size, expected, reason)| {
			assert_eq!(format!("{}", size), expected, "{reason}");
		});
	}

	#[test]
	fn log_format() {
		#[rustfmt::skip]
		[
			(SSize(0),                 "0 B",      "zero"),
			(SSize(1),                 "1 B",      "single byte"),
			(SSize(512),               "512 B",    "half KiB"),
			(SSize(1000),              "1 kB",     "exact decimal kB"),
			(SSize(1024),              "1 KiB",    "exact binary KiB"),
			(SSize(2000),              "2 kB",     "exact decimal 2kB"),
			(SSize(2048),              "2 KiB",    "exact binary 2KiB"),
			(SSize(1536),              "1.50 KiB", "fractional binary"),
			(SSize(1_000_000),         "1 MB",     "exact decimal MB"),
			(SSize(1_048_576),         "1 MiB",    "exact binary MiB"),
			(SSize(1_073_741_824),     "1 GiB",    "exact binary GiB"),
			(SSize(1_099_511_627_776), "1 TiB",    "exact binary TiB"),
		]
		.into_iter()
		.for_each(|(size, expected, reason)| {
			assert_eq!(format_logged(size), expected, "{reason}");
		});
	}

	#[test]
	fn errors() {
		#[rustfmt::skip]
		[
			("",     "empty string"),
			("abc",  "no digits"),
			("1xyz", "garbage suffix"),
			("1XB",  "unknown prefix X"),
			("1KiX", "unknown modifier X"),
			("-1",   "negative number"),
			("1k",   "lowercase single k (ambiguous)"),
		]
		.into_iter()
		.for_each(|(input, reason)| {
			assert!(SSize::from_str(input).is_err(), "{reason}");
		});
	}
}
