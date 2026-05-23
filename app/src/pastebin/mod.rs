use std::sync::Arc;

mod error;

use std::io;

pub use error::{Error, Result};
use rslog::{attrs, error, info};
use tokio::{
	io::{AsyncReadExt, AsyncWriteExt},
	net::{tcp::OwnedWriteHalf, TcpListener, TcpStream},
};

use crate::{
	config::{logging::ERR_KEY, Config},
	storage::{model::PasteInput, Storage},
	util::{decompress, SlugGenerator},
};

//                  data    over-the-limit
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
			let cfg = self.cfg.clone();

			tokio::spawn(async move {
				let result: Result<()> = async {
					let RawInput(raw, busted) =
						Self::recv_paste(max_buffer, &mut sock).await?;
          let paste = PasteInput::from(raw);

					let w = sock.into_split().1;

					if let Some(slug) = Self::find_match(&db, &paste).await? {
						// TODO: I'm on the fence about wether or not to include this extra.
						// If it's not the same author uploading twice, I'm potentially
						// giving info about someone.
						let msg =
							Some("It apears this has already been uploaded at some point. This the url from the original upload.\n");
						Self::return_url(w, &cfg, slug, msg).await?;
						return Ok(());
					}

          let msg = if busted {
            Some("It looks like what you uploaded was a bit too big and had to get cut short.\nSorry Eh...\n")
          } else {
              None
          };

					let slug = slugs.gen_unique(&db).await?;
					let compressed = paste.try_into_compressed_paste(slug.clone())?;
					let _created = db.queries.insert(&compressed).await?;
					Self::return_url(w, &cfg, slug, msg).await?;

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

	async fn return_url(
		mut w: OwnedWriteHalf,
		cfg: &Config,
		slug: String,
		extra: Option<&str>,
	) -> Result<()> {
		let extra = if let Some(v) = extra { v } else { "" };
		let mut msg: String =
			format!("{}/{}\n{}", cfg.server.public_addr, slug, extra);
		w.write_buf(&mut msg.as_bytes()).await?;
		Ok(())
	}

	async fn recv_paste(
		buf_max: usize,
		sock: &mut TcpStream,
	) -> Result<RawInput> {
		let mut output = String::with_capacity(BUFFER_CAPACITY);
		let mut buf: Box<[u8]> = vec![0; BUFFER_CAPACITY].into();
		let mut overflowed = false;

		// First read: blocking, wait for initial data
		let n = sock.read(&mut buf).await?;
		if n == 0 {
			return Ok(RawInput(output, false));
		}
		{
			let remainder = buf_max - output.len();
			let txt = String::from_utf8_lossy(&buf[..n]);
			if n > remainder {
				output.push_str(&txt[..remainder]);
				return Ok(RawInput(output, true));
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
						overflowed = true;
						break;
					} else {
						output.push_str(&txt);
					}
				}
				Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => break,
				Err(_) => break,
			}
		}

		Ok(RawInput(output, overflowed))
	}
}
