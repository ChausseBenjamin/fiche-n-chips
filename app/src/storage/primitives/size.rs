use sqlx::FromRow;

#[derive(FromRow)]
pub struct PageCounts {
	pub page_count:     i64,
	pub freelist_count: i64,
}

impl PageCounts {
	pub fn physical(&self, page_size: u64) -> u64 {
		u64::try_from(self.page_count).unwrap_or(0).saturating_mul(page_size)
	}

	pub fn live(&self, page_size: u64) -> u64 {
		u64::try_from(self.page_count.saturating_sub(self.freelist_count))
			.unwrap_or(0)
			.saturating_mul(page_size)
	}
}

/// Result of a database trim pass.
#[derive(Debug, Clone, Copy)]
pub struct TrimReport {
	// Rows deleted
	pub deleted:  u64,
	// Bytes
	pub before:   u64,
	pub after:    u64,
	// Whether a full `VACUUM` ran.
	pub vacuumed: bool,
}
