use std::time::SystemTime;

use sqlx;

use crate::util::{from_unix_timestamp, unix_secs_floor};

#[derive(Debug, Clone, Copy, sqlx::Type)]
#[sqlx(transparent)]
pub struct SystemTimeExt(i64);

impl From<SystemTimeExt> for SystemTime {
	fn from(t: SystemTimeExt) -> Self { from_unix_timestamp(t.0) }
}

impl From<SystemTime> for SystemTimeExt {
	fn from(t: SystemTime) -> Self { Self(unix_secs_floor(t)) }
}
