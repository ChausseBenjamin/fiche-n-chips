use std::time::{Duration, SystemTime, UNIX_EPOCH};

pub fn from_unix_timestamp(v: i64) -> SystemTime {
	if v >= 0 {
		UNIX_EPOCH + Duration::from_secs(v.unsigned_abs())
	} else {
		UNIX_EPOCH - Duration::from_secs(v.unsigned_abs())
	}
}
