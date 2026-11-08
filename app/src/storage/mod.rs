use std::borrow::Cow;
use std::path::Path;
use std::sync::Arc;
use std::time::{Duration, SystemTime};

use rslog::{attrs, debug, error, info, src, warn};
use sqlx::sqlite::SqlitePool;
use sqlx::{QueryBuilder, query_file_as, query_file_scalar};
use tokio::sync::Mutex;

use crate::config::logging::{COUNT_KEY, DESC_KEY, ERR_KEY};
use crate::domain::{
	Brief, EntryPacked, Naming, RecordPacked, RouteKind, SHOTGUN_ATTEMPTS,
	SHOTGUN_SHELL, Slug, SlugGenerator, UploadPending, UploadReady,
};
use crate::storage::primitives::Payload;
use crate::util::unix_secs_ceil;

mod error;
mod primitives;
mod setup;

pub use error::{Error, Result};
use primitives::{Meta, PageCounts, Route, RowKind};

pub struct Storage {
	db:        SqlitePool,
	shrink:    Mutex<()>,
	page_size: u64,
}

const SHRINK_BATCH: i64 = 512;

impl Storage {
	pub async fn sync_reserved_routes(
		&self,
		paths: impl IntoIterator<Item = &Path>,
	) -> Result<u64> {
		let ids: Vec<Cow<'_, str>> =
			paths.into_iter().map(Path::to_string_lossy).collect();

		if ids.is_empty() {
			error(
				"no http paths are reserved",
				attrs!(
					DESC_KEY => "the server should accept/serve stuff but getting this error indicates api endpoints aren't protected and no homepages are being hosted...",
				),
			);
			// Return Ok to remain resilient and serve content.
			// Best we can do is log the error...
			return Ok(0);
		}

		let mut tx = self.db.begin().await?;

		let mut insert = QueryBuilder::new(
			"INSERT OR IGNORE INTO routes (id, kind, hash, content) ",
		);
		insert.push_values(ids.iter(), |mut b, id| {
			b.push_bind(id.as_ref())
				.push_bind(i64::from(RowKind::Reserved))
				.push_bind(None::<i64>)
				.push_bind(None::<Vec<u8>>);
		});
		let inserted = insert.build().execute(&mut *tx).await?.rows_affected();

		let mut delete = QueryBuilder::new("DELETE FROM routes WHERE kind = ");
		delete.push_bind(i64::from(RowKind::Reserved));
		delete.push(" AND id NOT IN (");
		let mut separated = delete.separated(", ");
		for id in &ids {
			separated.push_bind(id.as_ref());
		}
		separated.push_unseparated(")");
		drop(separated);
		let deleted = delete.build().execute(&mut *tx).await?.rows_affected();

		tx.commit().await?;

		Ok(inserted.saturating_add(deleted))
	}

	pub async fn delete(&self, brief: Brief) -> Result<Brief> {
		let id = brief.slug.as_str();
		let meta = query_file_as!(Meta, "src/storage/statements/delete.sql", id)
			.fetch_one(&self.db)
			.await?;
		Ok(Brief::from(meta))
	}

	pub async fn exists(&self, id: &Slug) -> Result<bool> {
		Ok(
			query_file_scalar!("src/storage/statements/exists.sql", id.as_str())
				.fetch_one(&self.db)
				.await?,
		)
	}

	pub async fn get(&self, id: &Path) -> Result<Option<EntryPacked>> {
		let payload = query_file_as!(
			Payload,
			"src/storage/statements/get.sql",
			id.to_string_lossy()
		)
		.fetch_optional(&self.db)
		.await?;
		payload.map(EntryPacked::try_from).transpose()
	}

	pub async fn get_brief(&self, id: &Path) -> Result<Brief> {
		let meta = query_file_as!(
			Meta,
			"src/storage/statements/get_brief.sql",
			id.to_string_lossy(),
		)
		.fetch_one(&self.db)
		.await?;
		Ok(meta.into())
	}

	pub async fn get_created_between(
		&self,
		from: SystemTime,
		to: SystemTime,
	) -> Result<Vec<Brief>> {
		Ok(
			query_file_as!(
				Meta,
				"src/storage/statements/get_created_between.sql",
				unix_secs_ceil(from),
				unix_secs_ceil(to),
			)
			.fetch_all(&self.db)
			.await?
			.into_iter()
			.map(Brief::from)
			.collect(),
		)
	}

	pub async fn insert(&self, submission: UploadReady) -> Result<Brief> {
		let UploadReady {
			naming,
			content,
			checksum,
		} = submission;
		let kind: RowKind = RouteKind::from(&content).into();
		let hash = checksum.map(i64::from);
		let content = content.into_inner(); // Alloc-free move
		match naming {
			Naming::Fixed(slug) => self.insert_fixed(slug, hash, kind, content).await,
			Naming::Generated => self.insert_generated(hash, kind, content).await,
		}
	}

	async fn insert_fixed(
		&self,
		id: Slug,
		hash: Option<i64>,
		kind: RowKind,
		content: Box<[u8]>,
	) -> Result<Brief> {
		let meta = query_file_as!(
			Meta,
			"src/storage/statements/insert_fixed.sql",
			id.as_str(),
			i64::from(kind),
			hash,
			content,
		)
		.fetch_optional(&self.db)
		.await?
		// `ON CONFLICT ... DO NOTHING`: only 1 insert can
		// win if two concurrent requests want the same slug.
		// NOTHING => detect if we lost the race (we get None instead of a brief)
		.ok_or(Error::SlugTaken(id))?;
		Ok(Brief::from(meta))
	}

	#[rustfmt::skip]
	async fn insert_generated(
		&self,
		hash: Option<i64>,
		kind: RowKind,
		content: Box<[u8]>,
	) -> Result<Brief> {
		let shotgun = SlugGenerator::generate(&SHOTGUN_SHELL);
		let meta = query_file_as!(
			Meta, "src/storage/statements/insert_generated.sql",
      shotgun,
        i64::from(kind),
        hash,
        content)
            .fetch_optional(&self.db)
            .await?
            .ok_or(Error::Exhausted(SHOTGUN_ATTEMPTS))?;
		Ok(Brief::from(meta))
	}

	pub async fn potential_duplicates(
		&self,
		upload: &UploadPending,
	) -> Result<Vec<RecordPacked>> {
		let Some(hash) = upload.checksum() else {
			// hashes only exist for pastes; redirects are never deduplicated.
			return Ok(Vec::new());
		};
		query_file_as!(
			Route,
			"src/storage/statements/potential_duplicates.sql",
			i64::from(hash),
		)
		.fetch_all(&self.db)
		.await?
		.into_iter()
		.map(RecordPacked::try_from)
		.collect()
	}

	pub async fn purge_expired(&self, retention: Duration) -> Result<Vec<Brief>> {
		let retention = retention.as_secs().cast_signed();
		let expired = query_file_as!(
			Meta,
			"src/storage/statements/purge_expired.sql",
			&retention,
		)
		.fetch_all(&self.db)
		.await?;
		Ok(expired.into_iter().map(Brief::from).collect())
	}

	async fn size(&self) -> Result<PageCounts> {
		Ok(
			query_file_as!(PageCounts, "src/storage/statements/db_size.sql")
				.fetch_one(&self.db)
				.await?,
		)
	}

	pub async fn shrink_to_fit(&self, max_bytes: u64) -> Result<Vec<Brief>> {
		let _guard = self.shrink.lock().await;
		let mut deleted = Vec::new();

		if self.size().await?.live(self.page_size) <= max_bytes {
			return Ok(deleted);
		}

		loop {
			let mut tx = self.db.begin().await?;

			let batch = query_file_as!(
				Meta,
				"src/storage/statements/delete_oldest_batch.sql",
				SHRINK_BATCH,
			)
			.fetch_all(&mut *tx)
			.await?;

			if batch.is_empty() {
				tx.commit().await?;
				break;
			}

			// Read inside the transaction so the measurement includes this batch.
			let counts =
				query_file_as!(PageCounts, "src/storage/statements/db_size.sql")
					.fetch_one(&mut *tx)
					.await?;

			deleted.extend(batch.into_iter().map(Brief::from));
			tx.commit().await?;

			if counts.live(self.page_size) <= max_bytes {
				break;
			}
		}

		// Deletes only move pages to the freelist.
		if self.size().await?.physical(self.page_size) > max_bytes
			&& let Err(e) =
				sqlx::query("PRAGMA incremental_vacuum").execute(&self.db).await
		{
			warn(
				"failed to reclaim disk space after shrinking database",
				attrs!(ERR_KEY => e),
			);
		}

		Ok(deleted)
	}

	pub fn spawn_shrink(self: &Arc<Self>, max_bytes: u64) {
		let this = Arc::clone(self);
		tokio::spawn(async move {
			match this.shrink_to_fit(max_bytes).await {
				Ok(deleted) => {
					if deleted.is_empty() {
						debug("db is within size limit after insert", src!());
					} else {
						info(
							"freed space by deleting old entries after insert",
							attrs!(COUNT_KEY => deleted.len()),
						);
					}
				}
				Err(e) => {
					error("failed to shrink db after insert", attrs!(ERR_KEY => e));
				}
			}
		});
	}
}
