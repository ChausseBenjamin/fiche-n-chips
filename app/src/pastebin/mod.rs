use std::sync::{Arc, Mutex, PoisonError};

mod error;

use std::io;
use std::net::IpAddr;

pub use error::Result;
use rslog::{attrs, debug, error, info, src};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::time::{Duration, timeout};
use tokio_util::sync::CancellationToken;

use crate::anonymizer::Anonimizer;
use crate::config::Config;
use crate::config::logging::{ADDR_KEY, ERR_KEY, SLUG_KEY};
use crate::domain::{Brief, Resolved, Slug, UploadApproved, UploadPending};
use crate::limiter::{RateLimitStatus, RateLimiter};
use crate::storage::Storage;
use crate::ui;
use crate::util::{decompress, sorry};

//                  data    over-the-limit
pub struct RawInput(String, bool);

pub struct Pastebin {
	cfg:          Arc<Config>,
	db:           Arc<Storage>,
	anonimizer:   Arc<Anonimizer>,
	rate_limiter: Arc<Mutex<RateLimiter>>,
}

pub const BUFFER_CAPACITY: usize = 2048;

impl Pastebin {
	pub fn new(
		cfg: Arc<Config>,
		db: Arc<Storage>,
		anonimizer: Arc<Anonimizer>,
		rate_limiter: Arc<Mutex<RateLimiter>>,
	) -> Self {
		Self {
			cfg,
			db,
			anonimizer,
			rate_limiter,
		}
	}

	pub async fn listen(&self, shutdown: CancellationToken) -> io::Result<()> {
		let addr = format!(
			"{}:{}",
			self.cfg.server.listen_addr, self.cfg.server.paste_port,
		);
		let listener = TcpListener::bind(addr.clone()).await?;
		info("tcp listener started", attrs!( ADDR_KEY => addr,));

		#[allow(clippy::cast_possible_truncation)]
		let max_buffer = self.cfg.pastebin.max_buffer.get() as usize;
		let read_timeout = self.cfg.pastebin.read_timeout;
		loop {
			tokio::select! {
				result = listener.accept() => {
					let (mut sock, addr) = result?;

					let discrete = match addr.ip().to_canonical() {
						IpAddr::V4(v4) => self.anonimizer.cloak(&v4.octets()),
						IpAddr::V6(v6) => self.anonimizer.cloak(&v6.octets()),
					};
					let status = self.rate_limiter
						.lock()
						.unwrap_or_else(PoisonError::into_inner)
						.check_and_increment(discrete);
					if !matches!(status, RateLimitStatus::Allowed) {
							let msg = ui::format!(true, "{}\n{}", status, sorry());
							if let Err(e) = sock.write_all(msg.as_bytes()).await {
									error("failed to send rate limit response", attrs!(ERR_KEY => e));
							}
							continue;
					}

					let db = self.db.clone();
					let cfg = self.cfg.clone();

					tokio::spawn(async move {
						let result: Result<()> = async {
							let (mut reader, writer) = sock.into_split();
							let upload = Self::recv_paste(max_buffer, read_timeout, &mut reader).await?;
							let upload = match Self::resolve_paste(&db, &upload).await? {
										Resolved::Existing(brief) => {
								let msg = Some("It apears this has already been uploaded at some point.\n\
										Here is the url from the original upload.\n");
								Self::return_url(writer, &cfg, &brief.slug, msg).await?;
								debug("reused existing paste", attrs!(SLUG_KEY => brief.slug));
								return Ok(());
										},
										Resolved::Novel(upload) => upload,
								};

							let msg = if upload.len() > max_buffer {
								debug("received oversized paste", src!());
								Some(format!("It looks like what you uploaded was a bit too big and had to get cut short.\n{}", sorry()))
							} else {
									None
							};
							let ready = upload.prepare()?;

							let Brief{slug,kind: _,created: _} = db.insert(ready).await?;
							Self::return_url(writer, &cfg, &slug, msg.as_deref()).await?;
							debug("created new paste", attrs!(SLUG_KEY => slug));

							Ok(())
						}
						.await;

						if let Err(e) = result {
							error("failed to handle paste", attrs!(ERR_KEY => e));
						}
					});
				}
				() = shutdown.cancelled() => {
					info("pastebin server shutting down", src!());
					break;
				}
			}
		}
		Ok(())
	}

	async fn resolve_paste(
		db: &Storage,
		paste: &UploadPending,
	) -> Result<Resolved> {
		let candidates = db.potential_duplicates(paste).await?;
		Ok(paste.resolve(candidates)?)
	}

	async fn return_url(
		mut w: OwnedWriteHalf,
		cfg: &Config,
		slug: &Slug,
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
	) -> Result<UploadPending> {
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

		Ok(UploadPending::paste(output))
	}
}
