use std::sync::Arc;

mod error;

use std::io;

pub use error::{Error, Result};
use rslog::{attrs, error, info};
use tokio::{
	io::AsyncReadExt,
	net::{tcp::OwnedWriteHalf, TcpListener, TcpStream},
};

use crate::{
	config::{logging::ERR_KEY, Config},
	storage::{model::PasteInput, Storage},
	util::{decompress, SlugGenerator},
};

//                  data    overflowed
pub struct RawInput(String, bool);

pub struct Pastebin {
	cfg: Arc<Config>,
	db: Arc<Storage>,
	slugs: Arc<SlugGenerator>,
}

pub const BUFFER_CAPACITY: usize = 2048;

impl Pastebin {
	pub fn new(cfg: Arc<Config>, db: Arc<Storage>) -> Self {
		let slugs = Arc::from(SlugGenerator::new(cfg.management.slug_length));
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
			let (mut sock, _addr) = listener.accept().await?;
			let db = self.db.clone();
			let slugs = self.slugs.clone();

			tokio::spawn(async move {
				let result: Result<()> = async {
					let paste: PasteInput =
						Self::recv_paste(max_buffer, &mut sock).await?.into();

					let w = sock.into_split().1;

					if let Some(slug) = Self::find_match(&db, &paste).await? {
						Self::return_url(w, slug).await?;
						return Ok(());
					}

					let slug = slugs.gen_unique(&db).await?;
					let compressed = paste.try_into_compressed_paste(slug)?;
					let _created_on = db.queries.insert(&compressed).await?;

					// TODO: Return the url for hosted paste
					Ok(())
				}
				.await;

				if let Err(e) = result {
					error("failed to handle paste", attrs!(ERR_KEY => e));
				}
			});
		}
	}

	async fn find_match(
		db: &Storage,
		paste: &PasteInput,
	) -> Result<Option<String>> {
		let hash_matches = db.queries.get_slugs_by_hash(paste.hash()).await?;
		for p in hash_matches {
			let decompressed = decompress(p.content)?;
			if paste.content() == decompressed {
				return Ok(Some(p.slug));
			}
		}
		Ok(None)
	}

	async fn return_url(w: OwnedWriteHalf, slug: String) -> Result<()> {
		todo!()
	}

	async fn recv_paste(
		buf_max: usize,
		sock: &mut TcpStream,
	) -> Result<RawInput> {
		let mut output = String::with_capacity(BUFFER_CAPACITY);
		let mut buf: Box<[u8]> = vec![0; BUFFER_CAPACITY].into();

		// First read: blocking, wait for initial data
		let n = sock.read(&mut buf).await?;
		if n == 0 {
			return Ok(output);
		}
		{
			let remainder = buf_max - output.len();
			let txt = String::from_utf8_lossy(&buf[..n]);
			if n > remainder {
				output.push_str(&txt[..remainder]);
				return Ok(output);
			}
			output.push_str(&txt);
		}

		// Non-blocking reads: drain any remaining data in the kernel buffer.
		// When the client is done sending, try_read returns WouldBlock and we stop.
		loop {
			match sock.try_read(&mut buf) {
				Ok(n) => {
					let remainder = buf_max - output.len();
					let txt = String::from_utf8_lossy(&buf[..n]);
					if n > remainder {
						output.push_str(&txt[..remainder]);
						break;
					} else {
						output.push_str(&txt);
					}
				}
				Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => break,
				Err(_) => break,
			}
		}

		Ok(output)
	}
}
