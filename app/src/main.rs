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

use actix_web::{web, App, HttpServer};
use clap::Parser;
use config::logging::ERR_KEY;
use config::Config;
use rslog::{attrs, debug, error, info, src, Logger};

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

	let pb = Pastebin::new(cfg.clone(), storage.clone());
	let http = Arc::new(http::Server::new(
		cfg.clone(),
		storage.clone(),
		Arc::from(reserved),
	));

	let (_pb, _http) = tokio::join!(
		pb.listen(),
		http.listen(),
		// TODO: async scheduled task that manages cleanups
	);

	Ok(())
}
