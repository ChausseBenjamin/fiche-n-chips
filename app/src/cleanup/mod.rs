use std::sync::Arc;
use std::time::SystemTime;

use humantime::format_duration;
use rslog::{attrs, debug, error, info, src};
use tokio::time;
use tokio_util::sync::CancellationToken;

use crate::config::logging::ERR_KEY;
use crate::config::Config;
use crate::storage::Storage;

pub struct Cleaner {
	cfg: Arc<Config>,
	db:  Arc<Storage>,
}

impl Cleaner {
	pub fn new(cfg: Arc<Config>, db: Arc<Storage>) -> Self { Self { cfg, db } }

	pub async fn autoclean(self: Arc<Self>, shutdown: CancellationToken) {
		debug(
			"cleaner initialization",
			attrs!(
				"cleanup_interval" => format_duration(self.cfg.management.cleanup_interval),
				"retention" => format_duration(self.cfg.management.retention),
			),
		);
		let mut interval = time::interval(self.cfg.management.cleanup_interval);

		loop {
			tokio::select! {
				biased;
				() = shutdown.cancelled() => break,
				_ = interval.tick() => {},
			}
			match self.db.queries.del_expired(self.cfg.management.retention).await {
				Ok(v) => {
					if !v.is_empty() {
						let now = SystemTime::now();
						let routes: String = v
							.iter()
							.map(|route| format!("`{}`", route.id))
							.collect::<Vec<String>>()
							.join(",");
						let (oldest, youngest) = v.iter().fold(
							(&v[0].created, &v[0].created),
							|(min, max), route| {
								(
									if route.created < *min { &route.created } else { min },
									if route.created > *max { &route.created } else { max },
								)
							},
						);
						info(
							"cleaned up expired entries",
							attrs!(
								"routes" => routes,
								"oldest" => format_duration(now.duration_since(*oldest).unwrap_or_default()),
								"youngest" => format_duration(now.duration_since(*youngest).unwrap_or_default()),
							),
						);
					} else {
						debug("no expired entries to clean", src!());
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

			match self.db.shrink_to_fit(self.cfg.management.db_size.get()).await {
				Ok(v) => {
					if !v.is_empty() {
						info(
							"freed space by deleting old entries",
							attrs!("count" => v.len()),
						);
					} else {
						debug("db is within size limit", src!());
					}
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
