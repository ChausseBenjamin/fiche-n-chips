mod config;
mod pastebin;
mod storage;
mod util;

use std::{fmt::format, io::Result, net::SocketAddr, sync::Arc};

use actix_web::{get, web, App, HttpServer, Responder};
use clap::Parser;
use rslog::{attrs, debug, info, src, Logger};
use tokio::{
	io::{AsyncReadExt, AsyncWriteExt},
	net::TcpListener,
};

use crate::pastebin::Pastebin;

#[tokio::main]
async fn main() -> Result<()> {
	let cfg = Arc::from(config::Config::parse());
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

	let pb = Pastebin::new(cfg.clone());

	pb.listen().await;

	Ok(())
}
