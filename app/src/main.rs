#![feature(time_systemtime_limits, random, core_float_math)]
mod config;
mod pastebin;
mod reserved;
mod storage;
mod util;

use std::{fmt::format, io::Result, net::SocketAddr, sync::Arc};

use clap::Parser;
use config::{logging::ERR_KEY, Config};
use rslog::{attrs, debug, error, info, src, Logger};

use crate::{pastebin::Pastebin, storage::Storage};

#[tokio::main]
async fn main() -> Result<()> {
	let cfg = Arc::from(Config::parse());
	let logger: Logger = TryFrom::try_from(&cfg.logging)?;
	rslog::set_default(logger);
	debug(
		"logger configured",
		attrs!(
						"level" => cfg.logging.level.clone(),
						"format" => cfg.logging.format.clone(),
						"output" => cfg.logging.output.clone(),
						"trace" => cfg.logging.trace.clone(),
		),
	);
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

	let pb = Pastebin::new(cfg.clone(), storage);

	pb.listen().await;

	Ok(())
}
