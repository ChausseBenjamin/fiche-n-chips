mod config;
mod pastebin;
mod storage;
mod util;

use std::{fmt::format, io::Result, net::SocketAddr};

use actix_web::{get, web, App, HttpServer, Responder};
use clap::Parser;
use rslog::{attrs, debug, info, Logger};
use tokio::{
	io::{AsyncReadExt, AsyncWriteExt},
	net::TcpListener,
};

#[tokio::main]
async fn main() -> Result<()> {
	let cfg = config::Config::parse();
	let logger: Logger = cfg.logging.try_into()?;
	rslog::set_default(logger);
	debug("logger configured".into(), attrs!());

	let listener = TcpListener::bind("127.0.0.1:8080").await?;

	loop {
		let (sock, addr) = listener.accept().await?;

		tokio::spawn(async move {
			let mut buf: Box<[u8]> = vec![0; 2048].into();
			let (mut r, mut w) = sock.into_split();
			let n = r.read(&mut buf).await.unwrap();
			info(
				"received tcp data".into(),
				attrs!("size" => n, "raw_buffer" => format!("{:?}",unsafe {str::from_utf8_unchecked(&buf)})),
			);
			w.write(&mut buf[0..n]).await.unwrap();
		});
	}

	Ok(())
}
