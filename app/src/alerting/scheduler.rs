use std::collections::BTreeSet;
use std::future::pending;
use std::rc::Rc;
use std::sync::Arc;
use std::time::{Duration, SystemTime};

use actix_rt::spawn;
use rslog::{attrs, error, warn};
use tokio::time::{MissedTickBehavior, interval, sleep_until};
use tokio_util::sync::CancellationToken;

use super::expiry::{ContentKind, ExpiryBracket};
use super::{Error, ExpiryMsg, MsgEmitter};
use crate::config::Config;
use crate::config::logging::ERR_KEY;
use crate::storage::Storage;
use crate::util::to_tokio_instant;

const REFILL_EVERY: Duration = Duration::from_mins(20);
const OVERLAP: Duration = Duration::from_mins(1);
const WINDOW: Duration = REFILL_EVERY.saturating_add(OVERLAP);

pub struct AlertScheduler {
	emitter:   Rc<MsgEmitter>,
	storage:   Arc<Storage>,
	retention: Duration,
	queue:     BTreeSet<(SystemTime, ExpiryMsg)>,
}

impl AlertScheduler {
	pub fn new(
		emitter: Rc<MsgEmitter>,
		storage: Arc<Storage>,
		cfg: &Config,
	) -> Self {
		let retention = cfg.management.retention;
		Self {
			emitter,
			storage,
			retention,
			queue: BTreeSet::new(),
		}
	}

	pub async fn run(mut self, shutdown: CancellationToken) {
		let mut refill = interval(REFILL_EVERY);
		refill.set_missed_tick_behavior(MissedTickBehavior::Delay);
		loop {
			let next = self.queue.first().map(|(t, _)| to_tokio_instant(*t));
			let due = async {
				match next {
					Some(at) => sleep_until(at).await,
					None => pending::<()>().await,
				}
			};
			tokio::select! {
					biased;
					() = shutdown.cancelled() => break,
					_ = refill.tick() => {
							if let Err(e) = self.refill(SystemTime::now()).await {
									error("alert refill failed", attrs!(
													ERR_KEY => e,
									))
							}
					}
					_ = due => self.fire_due(),
			}
		}
	}

	async fn refill(&mut self, now: SystemTime) -> Result<(), Error> {
		for bracket in ExpiryBracket::iter() {
			let from = now + bracket.lead() - self.retention;
			let until = from + WINDOW;
			for r in self.storage.queries.get_created_between(from, until).await? {
				let kind: ContentKind = r.kind.try_into()?;
				let fire_at = r.created + self.retention - bracket.lead();
				self.queue.insert((fire_at, bracket.create_warning(r.id, kind)));
			}
		}
		Ok(())
	}

	fn fire_due(&mut self) {
		let now = SystemTime::now();
		while self.queue.first().is_some_and(|(t, _)| *t <= now) {
			let Some((_, msg)) = self.queue.pop_first() else {
				break;
			};
			let emitter = Rc::clone(&self.emitter);
			spawn(async move {
				if let Err(e) = emitter.emit(msg).await {
					warn(
						"emitting alert failed",
						attrs!(
										ERR_KEY => e,
						),
					);
				}
			});
		}
	}
}
