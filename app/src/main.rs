#![feature(random, core_float_math)]
mod config;
mod http;
mod pastebin;
mod reserved;
mod storage;
mod ui;
mod util;

use std::io::Result;
use std::sync::Arc;

use clap::Parser;
use config::logging::ERR_KEY;
use config::Config;
use rslog::{attrs, error, info, src, warn, Logger};
use tokio_util::sync::CancellationToken;

use crate::pastebin::Pastebin;
use crate::reserved::ReservedPathProvider;
use crate::storage::Storage;

#[tokio::main]
async fn main() -> Result<()> {
	let cfg = Arc::from(Config::parse());
	let logger: Logger = TryFrom::try_from(&cfg.logging)?;
	rslog::set_default(logger);
	info("starting server", src!());
	let reserved = ReservedPathProvider::new(&cfg);

	let storage =
		Arc::from(Storage::setup(cfg.clone()).await.unwrap_or_else(|e| {
			error(
				"failed to complete database setup",
				attrs!(
					ERR_KEY => e,
				),
			);
			panic!()
		}));
	let _changed_rows = storage
		.sync_reserved_routes(
			reserved.paths().into_iter().map(Into::into).collect(),
		)
		.await;

	let shutdown = CancellationToken::new();
	let shutdown_timeout = cfg.server.shutdown_timeout;

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

	let pb = Pastebin::new(cfg.clone(), storage.clone());
	let http = Arc::new(http::Server::new(
		cfg.clone(),
		storage.clone(),
		Arc::from(reserved),
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
				() = tokio::time::sleep(shutdown_timeout) => {
					warn("http server shutdown timeout exceeded, forcing shutdown", src!());
				}
			}
		}
		_ = &mut http_fut => {
			shutdown.cancel();
			tokio::select! {
				_ = &mut pb_fut => {},
				() = tokio::time::sleep(shutdown_timeout) => {
					warn("pastebin server shutdown timeout exceeded, forcing shutdown", src!());
				}
			}
		}
	}

	info("server shut down", src!());
	Ok(())
}
