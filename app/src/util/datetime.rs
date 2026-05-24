use std::time::{Duration, SystemTime, UNIX_EPOCH};

pub fn from_unix_timestamp(v: i64) -> SystemTime {
	UNIX_EPOCH + Duration::from_secs(v as u64)
}
