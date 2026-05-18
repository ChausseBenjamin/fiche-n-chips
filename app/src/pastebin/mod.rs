use std::net::TcpStream;

use rslog::{attrs, debug, error, warn};
use tokio::{
	io,
	io::AsyncReadExt,
	net::{tcp::OwnedReadHalf, TcpListener},
};

use crate::{
	config::{logging::ERR_KEY, Config, PastebinCfg},
	storage::Storage,
};

pub struct Pastebin<'a> {
	cfg: &'a Config,
	// db: &'b Storage,
}

pub const BUFFER_CAPACITY: usize = 2048;

impl<'a> Pastebin<'a> {
	pub fn new(cfg: &'a Config) -> Self {
		Self { cfg }
	}

	pub async fn listen(&self) -> io::Result<()> {
		let addr = format!(
			"{}:{}",
			self.cfg.server.listen_addr, self.cfg.server.paste_port,
		);
		let listener = TcpListener::bind(addr.clone()).await?;
		debug("tcp listener started".into(), attrs!( "address" => addr,));

		loop {
			// XXX: hash _addr for rate_limiting, blacklisting, maybe?
			let (sock, _addr) = listener.accept().await?;
			let (mut r, mut w) = sock.into_split();

			let upload =
				tokio::spawn(async move { self.receive_paste(r).await }).await?;
		}

		Ok(())
	}

	pub async fn receive_paste(&self, r: OwnedReadHalf) -> String {
		let buf_max = usize::from(self.cfg.pastebin.max_buffer);
		let mut output: String = Vec::with_capacity(BUFFER_CAPACITY).into();
		let mut buf: Box<[u8]> = vec![0; BUFFER_CAPACITY].into();

		loop {
			match r.read(&mut buf).await {
				Ok(0) => {
					// We can't discern between EOF and an error
					// Breaking silently is the safer/more graceful approach
					debug("incoming TCP upload potentially cut short".into(), attrs!());
					break;
				}
				Ok(n) => {
					let remainder = buf_max - output.len();
					let txt = String::from_utf8_lossy(&buf);
					if n > remainder {
						output.push_str(txt[0..remainder].into());
						break;
					} else {
						output.push_str(txt.into());
					}
				}
				Err(e) => {
					warn("failed to fully receive paste".into(), attrs!(ERR_KEY => e));
					break;
				}
			};
		}

		output
	}
}
