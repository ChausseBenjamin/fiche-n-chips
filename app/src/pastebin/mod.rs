use std::sync::Arc;

mod error;

use std::io;

pub use error::Result;
use rslog::{attrs, debug, error, info, src};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::net::TcpListener;
use tokio::time::{timeout, Duration};

use crate::config::logging::ERR_KEY;
use crate::config::Config;
use crate::storage::model::PasteInput;
use crate::storage::Storage;
use crate::util::{decompress, sorry, SlugGenerator};

//                  data    over-the-limit
pub struct RawInput(String, bool);

pub struct Pastebin {
	cfg:   Arc<Config>,
	db:    Arc<Storage>,
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

		#[allow(clippy::cast_possible_truncation)]
		let max_buffer = self.cfg.pastebin.max_buffer.get() as usize;
		let read_timeout = self.cfg.pastebin.read_timeout;
		loop {
			// XXX: hash _addr for rate_limiting, blacklisting, maybe?
			let (sock, _addr) = listener.accept().await?;
			let db = self.db.clone();
			let slugs = self.slugs.clone();
			let cfg = self.cfg.clone();

			tokio::spawn(async move {
				let result: Result<()> = async {
					let (mut reader, writer) = sock.into_split();
					let RawInput(raw, busted) =
						Self::recv_paste(max_buffer, read_timeout, &mut reader).await?;
          let paste = PasteInput::from(raw);

					if let Some(slug) = Self::find_match(&db, &paste).await? {
						// TODO: I'm on the fence about wether or not to include this extra.
						// If it's not the same author uploading twice, I'm potentially
						// giving info about someone.
						let msg =
							Some("It apears this has already been uploaded at some point.\nHere is the url from the original upload.\n");
						Self::return_url(writer, &cfg, slug.clone(), msg).await?;
            debug("reused existing paste", attrs!("slug" => slug));
						return Ok(());
					}

          let owned = if busted {
            debug("received oversized paste", src!());
            Some(format!("It looks like what you uploaded was a bit too big and had to get cut short.\n{}", sorry()))
          } else {
              None
          };
          let msg = owned.as_deref();

					let slug = slugs.gen_unique(&db).await?;
					let compressed = paste.try_into_compressed_paste(slug.clone())?;
					let _created = db.queries.insert(&compressed).await?;
					Self::return_url(writer, &cfg, slug.clone(), msg).await?;
          debug("created new paste", attrs!("slug" => slug));

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
		let extra = extra.unwrap_or_default();
		let msg: String = format!("{}/{}\n{}", cfg.server.public_addr, slug, extra);
		w.write_buf(&mut msg.as_bytes()).await?;
		Ok(())
	}

	async fn recv_paste(
		buf_max: usize,
		read_timeout: Duration,
		reader: &mut OwnedReadHalf,
	) -> Result<RawInput> {
		let mut output = String::with_capacity(BUFFER_CAPACITY);
		let mut buf: Box<[u8]> = vec![0; BUFFER_CAPACITY].into();
		let mut overflowed = false;

		loop {
			let n = match timeout(read_timeout, reader.read(&mut buf)).await {
				Ok(Ok(0)) => break,
				Ok(Ok(n)) => n,
				_ => break,
			};
			let remainder = buf_max - output.len();
			let txt = String::from_utf8_lossy(&buf[..n]);
			if n > remainder {
				output.push_str(&txt[..remainder]);
				overflowed = true;
				break;
			}
			output.push_str(&txt);
		}

		Ok(RawInput(output, overflowed))
	}
}
