use std::collections::HashMap;
use std::fmt::{Display, Formatter, Result};
use std::time::{Duration, SystemTime};

use rslog::{attrs, error, warn};

use crate::anonymizer::DiscreteId;
use crate::config::logging::{
	CONCURRENT, DISCRETE_ID_KEY, LIMIT_KEY, RETRY_AFTER_SECS_KEY,
};
use crate::ui::{self, PlainFormat};
use crate::util::{EMPHASIS, IDENT};

const CONCURENT_LIMITED_MAX: usize = 100_000;

struct RateLimit {
	count: u64,
	reset: SystemTime,
}

pub enum RateLimitStatus {
	Allowed,
	Denied { limit: u16, retry_after: Duration },
	Overload,
}

impl RateLimitStatus {
	// Single source of truth for the user-facing message. The transport picks
	// styled (ANSI) vs plain via how it renders this type; the wording and
	// emphasis live here, next to the limiter.
	fn render(&self, styled: bool) -> String {
		match self {
			Self::Allowed => String::new(),
			Self::Denied { limit, retry_after } => ui::format!(
				styled,
				"You've reached the limit ({}).\nPlease try again in {}.",
				ui::string(limit, IDENT),
				ui::string(humantime::format_duration(*retry_after), EMPHASIS),
			),
			Self::Overload => {
				"The server is swamped right now, please try again later.".to_string()
			}
		}
	}
}

impl Display for RateLimitStatus {
	fn fmt(&self, f: &mut Formatter<'_>) -> Result {
		f.write_str(&self.render(true))
	}
}

// `plain!`/`ui::format!` prefer this over `Display`, so non-ANSI callers get
// the same wording without escape codes.
impl PlainFormat for RateLimitStatus {
	fn fmt_plain(&self, f: &mut Formatter<'_>) -> Result {
		f.write_str(&self.render(false))
	}
}

pub struct RateLimiter {
	data: HashMap<DiscreteId, RateLimit>,
	max_requests: u16,
	period: Duration,
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
		let concurrent = self.data.len();
		match (self.data.get_mut(&id), concurrent) {
			(Some(entry), _) => {
				if now >= entry.reset {
					entry.count = 1;
					entry.reset = now + self.period;
					RateLimitStatus::Allowed
				} else if entry.count < u64::from(self.max_requests) {
					entry.count += 1;
					RateLimitStatus::Allowed
				} else {
					warn(
						"rate limit exceeded",
						attrs!(
                DISCRETE_ID_KEY => id,
                LIMIT_KEY => self.max_requests,
                RETRY_AFTER_SECS_KEY => entry.reset.duration_since(now)
                  .unwrap_or(Duration::ZERO)
                  .as_secs()),
					);
					RateLimitStatus::Denied {
						limit: self.max_requests,
						retry_after: entry
							.reset
							.duration_since(now)
							.unwrap_or(Duration::ZERO),
					}
				}
			}
			(None, concurrent) if concurrent >= CONCURENT_LIMITED_MAX => {
				error(
					"too many concurently limited users",
					attrs!(
                DISCRETE_ID_KEY => id,
                CONCURRENT => concurrent),
				);
				RateLimitStatus::Overload
			}
			(None, _) => {
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

	pub fn cleanup(&mut self) {
		let now = SystemTime::now();
		self.data.retain(|_, v| v.reset > now);
	}
}
