#![feature(random, core_float_math)]
#![feature(duration_constructors)]
mod alerting;
mod anonymizer;
mod cleanup;
mod config;
mod http;
mod limiter;
mod pastebin;
mod reserved;
mod storage;
mod ui;
mod util;

use std::io::Result;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

use clap::Parser;
use config::Config;
use config::logging::ERR_KEY;
use rslog::{Logger, attrs, error, info, src, warn};
use tokio::time::sleep;
use tokio_util::sync::CancellationToken;

use crate::alerting::{AlertScheduler, MsgEmitter};
use crate::anonymizer::Anonimizer;
use crate::cleanup::Cleaner;
use crate::limiter::RateLimiter;
use crate::pastebin::Pastebin;
use crate::reserved::ReservedPathProvider;
use crate::storage::Storage;
use crate::util::SlugGenerator;

#[tokio::main]
async fn main() -> Result<()> {
	let cfg = Arc::from(Config::parse());
	let logger: Logger = TryFrom::try_from(&cfg.logging)?;
	rslog::set_default(logger);
	info("starting server", src!());
	if rustls::crypto::ring::default_provider()
		.install_default()
		.is_err()
	{
		warn("rustls CryptoProvider was already installed", src!());
	}
	let reserved = ReservedPathProvider::new(&cfg);
	let slugs = Arc::from(SlugGenerator::new(cfg.management.slug_length));
	let anonimizer = Arc::from(Anonimizer::new(&cfg.management.seed_path)?);
	let rate_limiter = Arc::new(Mutex::new(RateLimiter::new(
		cfg.management.rate_limit,
		cfg.management.rate_limit_period,
	)));

	let storage =
		Arc::from(Storage::setup(cfg.clone()).await.unwrap_or_else(|e| {
			error("failed to complete database setup", attrs!( ERR_KEY => e));
			panic!()
		}));
	let _changed_rows = storage
		.sync_reserved_routes(
			reserved.paths().into_iter().map(Into::into).collect(),
		)
		.await;

	let cleaner = Arc::from(Cleaner::new(
		cfg.clone(),
		storage.clone(),
		rate_limiter.clone(),
	));

	let shutdown = CancellationToken::new();
	let shutdown_timeout = cfg.management.shutdown_timeout;

	let (sched_cfg, sched_db, sched_shutdown) =
		(cfg.clone(), storage.clone(), shutdown.clone());
	std::thread::spawn(move || {
		let emitter = match MsgEmitter::try_from(&sched_cfg.ntfy) {
			Ok(e) => Rc::new(e),
			Err(e) => {
				info(
					"alert scheduler disabled",
					attrs!(
					ERR_KEY => e,
					),
				);
				return;
			}
		};
		let scheduler = AlertScheduler::new(emitter, sched_db, &sched_cfg);
		let rt = actix_rt::Runtime::new()
			.expect("failed to build alert scheduler runtime");
		rt.block_on(scheduler.run(sched_shutdown));
	});

	let sig = shutdown.clone();
	tokio::spawn(async move {
		let mut term =
			tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
				.expect("failed to register SIGTERM handler");

		tokio::select! {
			_ = tokio::signal::ctrl_c() => {},
			_ = term.recv() => {},
		}

		info(
			"received shutdown signal, starting graceful shutdown",
			src!(),
		);
		sig.cancel();
	});

	tokio::spawn(cleaner.autoclean(shutdown.clone()));

	let pb = Pastebin::new(
		cfg.clone(),
		storage.clone(),
		slugs.clone(),
		anonimizer.clone(),
		rate_limiter.clone(),
	);
	let http = Arc::new(http::Server::new(
		cfg.clone(),
		storage.clone(),
		Arc::from(reserved),
		slugs,
		anonimizer,
		rate_limiter,
	));

	let pb_fut = pb.listen(shutdown.clone());
	let http_fut = http.listen(shutdown.clone());
	tokio::pin!(pb_fut);
	tokio::pin!(http_fut);

	tokio::select! {
		_ = &mut pb_fut => {
			shutdown.cancel();
			tokio::select! {
				_ = &mut http_fut => {},
				() = sleep(shutdown_timeout) => {
					warn("http server shutdown timeout exceeded, forcing shutdown", src!());
				}
			}
		}
		_ = &mut http_fut => {
			shutdown.cancel();
			tokio::select! {
				_ = &mut pb_fut => {},
				() = sleep(shutdown_timeout) => {
					warn("pastebin server shutdown timeout exceeded, forcing shutdown", src!());
				}
			}
		}
	}

	info("server shut down", src!());
	Ok(())
}
