use std::time::SystemTime;

#[repr(C)]
struct tm {
	tm_sec: i32,
	tm_min: i32,
	tm_hour: i32,
	tm_mday: i32,
	tm_mon: i32,
	tm_year: i32,
	tm_wday: i32,
	tm_yday: i32,
	tm_isdst: i32,
}

extern "C" {
	fn localtime_r(timep: *const i64, result: *mut tm) -> *mut tm;
}

pub(crate) fn format_time(time: SystemTime) -> String {
	let duration = time.duration_since(SystemTime::UNIX_EPOCH).unwrap_or_default();

	let secs = duration.as_secs() as i64;
	let millis = duration.subsec_nanos() / 1_000_000;

	let mut t: tm = unsafe { std::mem::zeroed() };
	unsafe {
		localtime_r(&secs as *const i64, &mut t as *mut tm);
	}

	let year = t.tm_year + 1900;
	let month = t.tm_mon + 1;
	let day = t.tm_mday;
	let hour = t.tm_hour;
	let min = t.tm_min;
	let sec = t.tm_sec;

	format!(
		"{:04}-{:02}-{:02} {:02}:{:02}:{:02}:{:03}",
		year, month, day, hour, min, sec, millis
	)
}
