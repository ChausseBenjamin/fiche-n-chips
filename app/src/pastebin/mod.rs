use std::sync::Arc;

mod error;

pub use error::Error;
use rslog::{attrs, debug, error, info, src, warn};
use tokio::{
	io::{self, AsyncReadExt},
	net::{
		tcp::{OwnedReadHalf, OwnedWriteHalf},
		TcpListener,
	},
};

use crate::{
	config::{logging::ERR_KEY, Config, PastebinCfg},
	storage::{model::HashedPaste, Storage},
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
				let paste: HashedPaste = match Self::recv_paste(max_buffer, r).await {
					Ok(v) => v.into(),
					Err(e) => todo!(),
				};

				if let Some(slug) = Self::find_match(db, paste).await? {
					Self::return_url(w, slug).await?;
				}
				// TODO: Detect potential collision
				if let Some(slug) = exists {
					Self::return_url(w, slug).await.unwrap();
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

	async fn find_match(
		db: Arc<Storage>,
		paste: HashedPaste,
	) -> Result<Option<String>, Error> {
		let hash_matches = db.queries.get_slugs_by_hash(paste.hash()).await?;
		for p in hash_matches {
			// TODO: decompress p.content and compare against the incoming paste
			let decompressed = "";
			if paste.content() == decompressed {
				return Ok(Some(p.slug));
			}
		}
		Ok(None)
	}

	async fn unique_slug(slugs: SlugGenerator, db: Storage) -> String {
		let mut output = slugs.generate();
		while db.queries.slug_exists(output).await? {
			debug(
				"collision while generating a slug name",
				attrs!( "slug" => output),
			);
			output = slugs.generate();
		}
		output
	}

	async fn return_url(
		mut w: OwnedWriteHalf,
		slug: String,
	) -> Result<(), Error> {
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
