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
