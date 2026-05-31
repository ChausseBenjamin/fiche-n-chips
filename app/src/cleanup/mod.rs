use std::sync::Arc;
use std::time::SystemTime;

use humantime::format_duration;
use rslog::{attrs, error, info};
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
		let mut interval = time::interval(self.cfg.management.cleanup_interval);

		loop {
			tokio::select! {
				biased;
				_ = shutdown.cancelled() => break,
				_ = interval.tick() => {},
			}
			match self.db.queries.del_expired(self.cfg.management.retention).await {
				Ok(mut v) => {
					if !v.is_empty() {
						let now = SystemTime::now();
						let routes: String = v
							.iter()
							.map(|route| format!("`{}`", route.id))
							.collect::<Vec<String>>()
							.join(",");
						v.sort_by_key(|route| route.created);
						let oldest = &v[0].created;
						let youngest = &v[v.len() - 1].created;
						info(
							"cleaned up expired entries",
							attrs!(
								"routes" => routes,
								"oldest" => format_duration(now.duration_since(*oldest).unwrap_or_default()),
								"youngest" => format_duration(now.duration_since(*youngest).unwrap_or_default()),
							),
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
		}
	}
}
