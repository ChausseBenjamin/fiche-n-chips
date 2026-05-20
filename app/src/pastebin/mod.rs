use std::{net::TcpStream, sync::Arc};

use rslog::{attrs, debug, error, info, src, warn};
use tokio::{
	io::{self, AsyncReadExt},
	net::{
		tcp::{OwnedReadHalf, OwnedWriteHalf},
		TcpListener,
	},
};
use xxhash_rust::xxh3;

use crate::{
	config::{logging::ERR_KEY, Config, PastebinCfg},
	storage::Storage,
	util::SlugGenerator,
};

pub struct Pastebin {
	cfg: Arc<Config>,
	db: Arc<Storage>,
	slugs: Arc<SlugGenerator>,
}

pub const BUFFER_CAPACITY: usize = 2048;

impl Pastebin {
	pub fn new(cfg: Arc<Config>, db: Arc<Storage>) -> Self {
		let slugs = Arc::from(SlugGenerator::new(cfg.pastebin.slug_length));
		Self { cfg, db, slugs }
	}

	pub async fn listen(&self) -> io::Result<()> {
		let addr = format!(
			"{}:{}",
			self.cfg.server.listen_addr, self.cfg.server.paste_port,
		);
		let listener = TcpListener::bind(addr.clone()).await?;
		info("tcp listener started", attrs!( "address" => addr,));

		let max_buffer = self.cfg.pastebin.max_buffer as usize;
		loop {
			// XXX: hash _addr for rate_limiting, blacklisting, maybe?
			let (sock, _addr) = listener.accept().await?;
			let (mut r, mut w) = sock.into_split();
			let mut db = self.db.clone();

			tokio::spawn(async move {
				let content = Self::recv_paste(max_buffer, r).await?;
				let hash = xxh3::xxh3_64(content.as_bytes());
				let exists = db.get_slug_by_hash(hash).await.unwrap();
				// TODO: Detect potential collision
				if !exists.is_empty() {
					Self::reply_url(w, exists).await.unwrap();
					// TODO: Return pre-existing paste url from corresponding hash
				}
				// TODO: Compress data
				// TODO: Insert in DB
				// TODO: Return the url for hosted paste
				Ok::<(), ()>(())
			});
		}

		Ok(())
	}

	async fn reply_url(mut w: OwnedWriteHalf, slug: String) -> Result<(), ()> {
		todo!()
	}

	async fn recv_paste(
		buf_max: usize,
		mut r: OwnedReadHalf,
	) -> Result<String, ()> {
		let mut output: String = String::with_capacity(BUFFER_CAPACITY);
		let mut buf: Box<[u8]> = vec![0; BUFFER_CAPACITY].into();

		loop {
			match r.read(&mut buf).await {
				Ok(0) => {
					// We can't discern between EOF and an error
					// Breaking silently is the safer/more graceful approach
					debug("incoming TCP upload potentially cut short", src!());
					break;
				}
				Ok(n) => {
					let remainder = buf_max - output.len();
					let txt = String::from_utf8_lossy(&buf[..n]);
					if n > remainder {
						output.push_str(txt[0..remainder].into());
						break;
					} else {
						output.push_str(&txt);
					}
				}
				Err(e) => {
					warn("failed to fully receive paste", attrs!(ERR_KEY => e));
					return Err(());
				}
			};
		}

		Ok(output)
	}
}
