use std::sync::{Arc, Mutex, PoisonError};

mod error;

use std::net::IpAddr;
use std::{io, str};

pub use error::Result;
use rslog::{__log_kvpairs, Logger, attrs, info, src};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::time::{Duration, timeout};
use tokio_util::sync::CancellationToken;

use crate::anonymizer::Anonimizer;
use crate::config::Config;
use crate::config::logging::{ADDR_KEY, DISCRETE_ID_KEY, ERR_KEY, SLUG_KEY};
use crate::domain::{Brief, Resolved, Slug, UploadPending};
use crate::limiter::{RateLimitStatus, RateLimiter};
use crate::storage::Storage;
use crate::ui;
use crate::util::sorry;

pub const DRAIN_BUDGET: Duration = Duration::from_millis(250);
pub const DRAIN_MAX_BYTES: usize = 64 * 1024;

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
					// TODO: clean up rslog api to avoid calling `__`-prefixed macros
					let mut log = rslog::with(__log_kvpairs!(DISCRETE_ID_KEY => discrete).to_vec());
					let status = self.rate_limiter
						.lock()
						.unwrap_or_else(PoisonError::into_inner)
						.check_and_increment(discrete);
					if !matches!(status, RateLimitStatus::Allowed) {
							let msg = ui::format!(true, "{}\n{}", status, sorry());
							if let Err(e) = sock.write_all(msg.as_bytes()).await {
									log.error("failed to send rate limit response", attrs!(ERR_KEY => e));
							}
							continue;
					}

					let db = self.db.clone();
					let cfg = self.cfg.clone();

					tokio::spawn(async move {
						if let Err(e) =
							Self::handle_upload(sock, &cfg, &db, max_buffer, read_timeout, &mut log).await
						{
							log.error("failed to handle paste", attrs!(ERR_KEY => e));
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

	async fn handle_upload(
		sock: TcpStream,
		cfg: &Config,
		db: &Arc<Storage>,
		max_buffer: usize,
		read_timeout: Duration,
		log: &mut Logger,
	) -> Result<()> {
		let (mut reader, mut writer) = sock.into_split();

		let (upload, truncated) =
			match Self::recv_paste(max_buffer, read_timeout, &mut reader).await? {
				Received::Rejected { at } => {
					let msg = ui::format!(
						true,
						"What you uploaded doesn't look like UTF-8 text \
						 (first bad byte at offset {}).\n\
						 Nothing was saved.\n{}",
						at,
						sorry()
					);
					Self::reply(&mut writer, &mut reader, &msg).await?;
					log.debug("rejected non-utf8 paste", attrs!(ERR_KEY => at));
					return Ok(());
				}
				Received::Accepted { content, truncated } => (content, truncated),
			};

		// Built before deduplication so the reused-paste branch can mention
		// that the upload was cut short too.
		let notice = truncated.then(|| {
			log.debug("received oversized paste", src!());
			format!(
				"It looks like what you uploaded was a bit too big and had to get cut \
				 short.\nOnly the first {} bytes were kept.\n{}",
				upload.len(),
				sorry()
			)
		});

		let candidates = db.potential_duplicates(&upload).await?;
		match upload.resolve(candidates)? {
			Resolved::Existing(brief) => {
				let mut msg = String::from(
					"It apears this has already been uploaded at some point.\n\
					 Here is the url from the original upload.\n",
				);
				if let Some(notice) = &notice {
					msg.push_str(notice);
				}
				Self::return_url(
					&mut writer,
					&mut reader,
					cfg,
					&brief.slug,
					Some(&msg),
				)
				.await?;
				log.debug("reused existing paste", attrs!(SLUG_KEY => brief.slug));
			}
			Resolved::Novel(upload) => {
				let ready = upload.prepare()?;
				let Brief { slug, .. } = db.insert(ready).await?;
				Self::return_url(
					&mut writer,
					&mut reader,
					cfg,
					&slug,
					notice.as_deref(),
				)
				.await?;
				log.debug("created new paste", attrs!(SLUG_KEY => slug));
			}
		}

		Ok(())
	}

	async fn return_url<W, R>(
		w: &mut W,
		r: &mut R,
		cfg: &Config,
		slug: &Slug,
		extra: Option<&str>,
	) -> Result<()>
	where
		W: AsyncWrite + Unpin,
		R: AsyncRead + Unpin,
	{
		let extra = extra.unwrap_or_default();
		let msg = format!("{}/{}\n{}", cfg.server.public_addr, slug, extra);
		Self::reply(w, r, &msg).await
	}

	/// Write `msg`, then half-close and briefly drain the socket.
	///
	/// The drain isn't cosmetic: a client that is still writing (which is the
	/// case for every paste that hit the size cap) leaves unread bytes in the
	/// receive buffer, and closing with data pending makes the stack send RST,
	/// which can discard `msg` before the client manages to read it.
	async fn reply<W, R>(w: &mut W, r: &mut R, msg: &str) -> Result<()>
	where
		W: AsyncWrite + Unpin,
		R: AsyncRead + Unpin,
	{
		w.write_all(msg.as_bytes()).await?;
		w.shutdown().await?;
		Self::drain(r).await;
		Ok(())
	}

	/// Discard pending input, bounded by [`DRAIN_BUDGET`] and
	/// [`DRAIN_MAX_BYTES`].
	async fn drain<R: AsyncRead + Unpin>(r: &mut R) {
		let mut sink = [0u8; BUFFER_CAPACITY];
		let mut budget = DRAIN_MAX_BYTES;
		let _ = timeout(DRAIN_BUDGET, async {
			while budget > 0 {
				match r.read(&mut sink).await {
					Ok(0) | Err(_) => break,
					Ok(n) => budget = budget.saturating_sub(n),
				}
			}
		})
		.await;
	}

	async fn recv_paste<R: AsyncRead + Unpin>(
		buf_max: usize,
		read_timeout: Duration,
		reader: &mut R,
	) -> Result<Received> {
		let limit = buf_max.saturating_add(1);
		let mut out: Vec<u8> = Vec::with_capacity(BUFFER_CAPACITY.min(limit));
		let mut buf: Box<[u8]> = vec![0; BUFFER_CAPACITY].into();
		let mut truncated = false;

		loop {
			let n = match timeout(read_timeout, reader.read(&mut buf)).await {
				Ok(Ok(0)) => break,
				Ok(Ok(n)) => n,
				_ => break,
			};
			let room = limit.saturating_sub(out.len());
			#[allow(clippy::indexing_slicing)] // take <= n; n comes from read()
			out.extend_from_slice(&buf[..n.min(room)]);
			if out.len() > buf_max {
				out.truncate(buf_max);
				truncated = true;
				break;
			}
		}

		// A byte-wise cut can land inside a multi-byte character. Back off to
		// the last boundary so an oversized-but-valid paste isn't reported as
		// bad UTF-8 because of where it happened to be cut.
		if truncated
			&& let Err(e) = str::from_utf8(&out)
			&& e.error_len().is_none()
		{
			out.truncate(e.valid_up_to());
		}

		if let Err(e) = str::from_utf8(&out) {
			return Ok(Received::Rejected {
				at: e.valid_up_to(),
			});
		}

		Ok(Received::Accepted {
			content: UploadPending::paste(out),
			truncated,
		})
	}
}

enum Received {
	Rejected {
		at: usize,
	},
	Accepted {
		content:   UploadPending,
		truncated: bool,
	},
}

#[cfg(test)]
mod tests {
	use std::io;
	use std::pin::Pin;
	use std::task::{Context, Poll};

	use tokio::io::{AsyncRead, AsyncReadExt, AsyncWriteExt, ReadBuf, duplex};
	use tokio::time::{Duration, timeout};

	use super::{BUFFER_CAPACITY, DRAIN_MAX_BYTES, Pastebin, Received};

	const FILL: [u8; BUFFER_CAPACITY] = [b'a'; BUFFER_CAPACITY];

	/// A reader that reports EOF after `max_reads`, so "did we stop early?"
	/// is observable without relying on an outer timeout that a reader which
	/// always returns data immediately could starve.
	struct Endless {
		reads:     usize,
		max_reads: usize,
	}

	impl Endless {
		fn new(max_reads: usize) -> Self {
			Self {
				reads: 0,
				max_reads,
			}
		}
	}

	impl AsyncRead for Endless {
		fn poll_read(
			self: Pin<&mut Self>,
			_cx: &mut Context<'_>,
			buf: &mut ReadBuf<'_>,
		) -> Poll<io::Result<()>> {
			let this = self.get_mut();
			if this.reads >= this.max_reads {
				return Poll::Ready(Ok(())); // EOF
			}
			this.reads = this.reads.saturating_add(1);
			let fill = buf.remaining().min(FILL.len());
			buf.put_slice(&FILL[..fill]);
			Poll::Ready(Ok(()))
		}
	}

	async fn recv(payload: &[u8], buf_max: usize) -> Received {
		let capacity = BUFFER_CAPACITY.max(payload.len().saturating_add(1));
		let (mut tx, mut rx) = duplex(capacity);
		tx.write_all(payload).await.expect("write payload");
		drop(tx);

		timeout(
			Duration::from_secs(2),
			Pastebin::recv_paste(buf_max, Duration::from_millis(50), &mut rx),
		)
		.await
		.expect("recv_paste must not block")
		.expect("recv_paste failed")
	}

	/// A multi-byte character split by the read buffer must survive intact.
	/// The old per-chunk `from_utf8_lossy` turned these into U+FFFD.
	#[tokio::test]
	async fn multibyte_character_split_across_reads_survives() {
		let mut payload = vec![b'a'; BUFFER_CAPACITY.saturating_sub(1)];
		payload.extend_from_slice("€".as_bytes()); // straddles the 2048 boundary
		payload.push(b'\n');

		let Received::Accepted { content, truncated } =
			recv(&payload, BUFFER_CAPACITY.saturating_mul(8)).await
		else {
			panic!("valid utf-8 must be accepted");
		};

		assert!(!truncated);
		assert_eq!(content.raw(), payload.as_slice());
	}

	#[tokio::test]
	async fn invalid_utf8_is_rejected() {
		let Received::Rejected { at } = recv(b"before\xffafter", 1024).await else {
			panic!("invalid utf-8 must not be stored");
		};

		assert_eq!(at, 6);
	}

	/// Cutting at the size limit must not split a character, otherwise the
	/// paste is refused as invalid UTF-8 instead of being reported as too big.
	#[tokio::test]
	async fn truncation_keeps_a_valid_utf8_prefix() {
		let payload = "é".repeat(6); // 12 bytes, 2 per character

		let Received::Accepted { content, truncated } =
			recv(payload.as_bytes(), 9).await
		else {
			panic!("oversized valid utf-8 must still be accepted");
		};

		assert!(truncated);
		assert_eq!(content.raw(), "é".repeat(4).as_bytes());
	}

	#[tokio::test]
	async fn reading_stops_at_the_cap() {
		let buf_max = 4096;
		// More data available than the cap allows; a correct read loop never
		// gets far enough to see the EOF.
		let mut reader = Endless::new(10);

		let got = timeout(
			Duration::from_secs(2),
			Pastebin::recv_paste(buf_max, Duration::from_millis(50), &mut reader),
		)
		.await
		.expect("recv_paste must not drain the whole stream")
		.expect("recv_paste failed");

		let Received::Accepted { content, truncated } = got else {
			panic!("an endless stream of 'a' is valid utf-8");
		};

		assert!(truncated);
		assert_eq!(content.len(), buf_max);
		assert_eq!(
			reader.reads,
			buf_max.saturating_add(1).div_ceil(BUFFER_CAPACITY)
		);
	}

	#[tokio::test]
	async fn reply_is_bounded_by_the_drain_budget() {
		let (mut w, mut peer) = duplex(256);
		// Keep far more data available than the drain is allowed to consume.
		let mut r = Endless::new(100);

		timeout(
			Duration::from_secs(2),
			Pastebin::reply(&mut w, &mut r, "hello\n"),
		)
		.await
		.expect("reply must not wait for the client to stop writing")
		.expect("reply failed");

		let mut got = String::new();
		peer.read_to_string(&mut got).await.expect("read reply");
		assert_eq!(got, "hello\n");
		assert!(
			r.reads <= DRAIN_MAX_BYTES.div_ceil(BUFFER_CAPACITY),
			"drain consumed {} reads, more than the {} byte cap",
			r.reads,
			DRAIN_MAX_BYTES
		);
	}
}
