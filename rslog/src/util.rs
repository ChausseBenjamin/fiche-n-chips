use std::time::SystemTime;

pub(crate) fn format_time(time: SystemTime) -> String {
	let duration = time.duration_since(SystemTime::UNIX_EPOCH).unwrap_or_default();
	let total_secs = duration.as_secs();
	let millis = duration.subsec_nanos() / 1_000_000;

	let secs_of_day = total_secs % 86400;
	let hour = secs_of_day / 3600;
	let minute = (secs_of_day % 3600) / 60;
	let second = secs_of_day % 60;

	let days = total_secs / 86400;
	let (year, month, day) = days_to_date(days);

	format!(
		"{:04}-{:02}-{:02} {:02}:{:02}:{:02}:{:03}",
		year, month, day, hour, minute, second, millis
	)
}

fn is_leap_year(y: i64) -> bool {
	(y % 4 == 0 && y % 100 != 0) || y % 400 == 0
}

fn days_to_date(days: u64) -> (i64, u32, u32) {
	let mut y = 1970i64;
	let mut d = days as i64;

	loop {
		let dim = if is_leap_year(y) { 366 } else { 365 };
		if d < dim {
			break;
		}
		d -= dim;
		y += 1;
	}

	const MONTH_DAYS: [u32; 12] = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
	let mut m = 0u32;
	loop {
		let dim = if m == 1 && is_leap_year(y) {
			29
		} else {
			MONTH_DAYS[m as usize]
		};
		if d < dim as i64 {
			break;
		}
		d -= dim as i64;
		m += 1;
	}

	(y, m + 1, (d + 1) as u32)
}
