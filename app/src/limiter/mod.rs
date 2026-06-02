use std::collections::HashMap;
use std::time::{Duration, SystemTime};

use rslog::{attrs, warn};

use crate::anonymizer::DiscreteId;

struct RateLimit {
	count:  u64,
	reset:  SystemTime,
}

pub enum RateLimitStatus {
	Allowed,
	Denied {
		limit:        u16,
		retry_after:  Duration,
	},
}

pub struct RateLimiter {
	data:          HashMap<DiscreteId, RateLimit>,
	max_requests:  u16,
	period:        Duration,
}

impl RateLimiter {
	pub fn new(max_requests: u16, period: Duration) -> Self {
		Self {
			data: HashMap::new(),
			max_requests,
			period,
		}
	}

	pub fn check_and_increment(&mut self, id: DiscreteId) -> RateLimitStatus {
		let now = SystemTime::now();
		if let Some(entry) = self.data.get_mut(&id) {
			if now >= entry.reset {
				entry.count = 1;
				entry.reset = now + self.period;
				RateLimitStatus::Allowed
			} else if entry.count < self.max_requests as u64 {
				entry.count += 1;
				RateLimitStatus::Allowed
			} else {
				warn(
					"rate limit exceeded",
					attrs!("discrete_id" => id, "limit" => self.max_requests, "retry_after_secs" => entry.reset.duration_since(now).unwrap_or(Duration::ZERO).as_secs()),
				);
				RateLimitStatus::Denied {
					limit: self.max_requests,
					retry_after: entry.reset.duration_since(now).unwrap_or(Duration::ZERO),
				}
			}
		} else {
			self.data.insert(
				id,
				RateLimit {
					count: 1,
					reset: now + self.period,
				},
			);
			RateLimitStatus::Allowed
		}
	}
}
