use std::time::{Duration, SystemTime, UNIX_EPOCH};

use tokio::time::Instant;

pub fn from_unix_timestamp(v: i64) -> SystemTime {
	if v >= 0 {
		UNIX_EPOCH + Duration::from_secs(v.unsigned_abs())
	} else {
		UNIX_EPOCH - Duration::from_secs(v.unsigned_abs())
	}
}

pub fn to_tokio_instant(t: SystemTime) -> Instant {
	let d = t.duration_since(SystemTime::now()).unwrap_or(Duration::ZERO);
	Instant::now() + d
}

pub fn unix_secs_floor(t: SystemTime) -> i64 {
	t.duration_since(UNIX_EPOCH)
		.map_or(0, |d| i64::try_from(d.as_secs()).unwrap_or(i64::MAX))
}

pub fn unix_secs_ceil(t: SystemTime) -> i64 {
	t.duration_since(UNIX_EPOCH).map_or(0, |d| {
		let secs = d.as_secs() + u64::from(d.subsec_nanos() > 0);
		i64::try_from(secs).unwrap_or(i64::MAX)
	})
}
