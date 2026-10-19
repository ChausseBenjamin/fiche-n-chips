use std::time::SystemTime;

const MONTH_DAYS: [u32; 12] = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];

pub fn format_time(time: SystemTime) -> String {
	let duration =
		time.duration_since(SystemTime::UNIX_EPOCH).unwrap_or_default();
	let total_secs = duration.as_secs();
	let millis = duration.subsec_millis();

	let secs_of_day = total_secs % 86400;
	let hour = secs_of_day / 3600;
	let minute = (secs_of_day % 3600) / 60;
	let second = secs_of_day % 60;

	let days = total_secs / 86400;
	let (year, month, day) = days_to_date(days);

	format!(
		"{year:04}-{month:02}-{day:02} {hour:02}:{minute:02}:{second:02}:{millis:03}"
	)
}

const fn is_leap_year(y: i64) -> bool {
	(y % 4 == 0 && y % 100 != 0) || y % 400 == 0
}

fn days_to_date(days: u64) -> (i64, u32, i64) {
	let mut y = 1970i64;
	let mut d = days.cast_signed();

	loop {
		let dim = if is_leap_year(y) { 366 } else { 365 };
		if d < dim {
			break;
		}
		d -= dim;
		y += 1;
	}

	let mut m = 0u32;
	loop {
		let dim =
			if m == 1 && is_leap_year(y) { 29 } else { MONTH_DAYS[m as usize] };
		if d < i64::from(dim) {
			break;
		}
		d -= i64::from(dim);
		m += 1;
	}

	(y, m + 1, d + 1)
}
