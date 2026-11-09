use std::sync::{Arc, Mutex, PoisonError};
use std::time::SystemTime;

use humantime::format_duration;
use rslog::{attrs, debug, error, info, src};
use tokio::time;
use tokio_util::sync::CancellationToken;

use crate::config::Config;
use crate::config::logging::{
	CLEANUP_INTERVAL_KEY, COUNT_KEY, ERR_KEY, OLDEST_KEY, RETENTION_KEY,
	ROUTES_KEY, YOUNGEST_KEY,
};
use crate::limiter::RateLimiter;
use crate::storage::Storage;

pub struct Cleaner {
	cfg:     Arc<Config>,
	db:      Arc<Storage>,
	limiter: Arc<Mutex<RateLimiter>>,
}

impl Cleaner {
	pub fn new(
		cfg: Arc<Config>,
		db: Arc<Storage>,
		limiter: Arc<Mutex<RateLimiter>>,
	) -> Self {
		Self { cfg, db, limiter }
	}

	pub async fn autoclean(self: Arc<Self>, shutdown: CancellationToken) {
		debug(
			"cleaner initialization",
			attrs!(
				CLEANUP_INTERVAL_KEY => format_duration(self.cfg.mgmt.cleanup_interval),
				RETENTION_KEY => format_duration(self.cfg.mgmt.retention)),
		);
		let mut interval = time::interval(self.cfg.mgmt.cleanup_interval);

		loop {
			tokio::select! {
				biased;
				() = shutdown.cancelled() => break,
				_ = interval.tick() => {},
			}
			self.limiter.lock().unwrap_or_else(PoisonError::into_inner).cleanup();
			match self.db.purge_expired(self.cfg.mgmt.retention).await {
				Ok(list) => {
					if list.is_empty() {
						debug("no expired entries to clean", src!());
					} else {
						let now = SystemTime::now();
						let routes: String = list
							.iter()
							.map(|brief| brief.slug.to_string())
							.collect::<Vec<String>>()
							.join(",");
						let (oldest, youngest) = list.iter().fold(
							(&list[0].created, &list[0].created),
							|(min, max), brief| {
								(
									if brief.created < *min { &brief.created } else { min },
									if brief.created > *max { &brief.created } else { max },
								)
							},
						);
						info(
							"cleaned up expired entries",
							attrs!(
								ROUTES_KEY => routes,
								OLDEST_KEY => format_duration(now.duration_since(*oldest).unwrap_or_default()),
								YOUNGEST_KEY => format_duration(now.duration_since(*youngest).unwrap_or_default())),
						);
					}
				}
				Err(e) => {
					error(
						"failure while cleaning up expired entries",
						attrs!(
							ERR_KEY => e,
						),
					);
				}
			}

			match self.db.shrink_to_fit(self.cfg.mgmt.db_size.get()).await {
				Ok(0) => {
					debug("db is within size limit", src!());
				}
				Ok(deleted) => {
					info(
						"freed space by deleting old entries",
						attrs!(COUNT_KEY => deleted),
					);
				}
				Err(e) => {
					error(
						"failure while shrinking database to max size",
						attrs!(ERR_KEY => e),
					);
				}
			}
		}
	}
}
