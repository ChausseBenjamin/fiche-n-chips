use std::borrow::Cow;
use std::path::Path;
use std::time::{Duration, SystemTime};

use rslog::{attrs, error, warn};
use sqlx::sqlite::SqlitePool;
use sqlx::{query_file, query_file_as, query_file_scalar};

use crate::config::logging::{DESC_KEY, ERR_KEY};
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
pub use primitives::TrimReport;
use primitives::{Meta, PageCounts, Route, RowKind};

pub struct Storage {
	db:        SqlitePool,
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
					DESC_KEY => "the server should accept/serve stuff but getting this error \
											 indicates api endpoints aren't protected and no homepages \
											 are being hosted...",
				),
			);
			// Ok to remain resilient and serve content. Best we can do is log...
			return Ok(0);
		}

		let mut tx = self.db.begin().await?;
		let mut inserted: u64 = 0;
		let deleted =
			query_file!("src/storage/statements/delete_all_reserved.sql",)
				.execute(&mut *tx)
				.await?
				.rows_affected();

		for id in &ids {
			let meta = query_file_as!(
				Meta,
				"src/storage/statements/insert_fixed.sql",
				id.as_ref(),
				i64::from(RowKind::Reserved),
				None::<i64>,
				None::<Vec<u8>>,
			)
			.fetch_optional(&mut *tx)
			.await?;
			if meta.is_some() {
				inserted += 1;
			}
		}

		tx.commit().await?;

		Ok(inserted.saturating_add(deleted))
	}

	/// Deletes a route by slug, returning the removed brief if it existed.
	pub async fn delete(&self, id: &Slug) -> Result<Option<Brief>> {
		Ok(
			query_file_as!(Meta, "src/storage/statements/delete.sql", id.as_str())
				.fetch_optional(&self.db)
				.await?
				.map(Brief::from),
		)
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

	/// Returns the brief for a route, or `None` if it does not exist.
	pub async fn get_brief(&self, id: &Path) -> Result<Option<Brief>> {
		Ok(
			query_file_as!(
				Meta,
				"src/storage/statements/get_brief.sql",
				id.to_string_lossy(),
			)
			.fetch_optional(&self.db)
			.await?
			.map(Brief::from),
		)
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

	async fn insert_generated(
		&self,
		hash: Option<i64>,
		kind: RowKind,
		content: Box<[u8]>,
	) -> Result<Brief> {
		let shotgun = SlugGenerator::generate(&SHOTGUN_SHELL);
		let meta = query_file_as!(
			Meta,
			"src/storage/statements/insert_generated.sql",
			shotgun,
			i64::from(kind),
			hash,
			content
		)
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
		let retention = i64::try_from(retention.as_secs()).unwrap_or(i64::MAX);
		let expired = query_file_as!(
			Meta,
			"src/storage/statements/purge_expired.sql",
			retention,
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

	pub async fn trim_to_spec(
		&self,
		cap: u64,
		target: u64,
	) -> Result<TrimReport> {
		let before = self.size().await?;
		let phys_before = before.physical(self.page_size);
		let mut report = TrimReport {
			deleted:  0,
			before:   phys_before,
			after:    phys_before,
			vacuumed: false,
		};

		if phys_before < cap {
			return Ok(report);
		}

		if before.live(self.page_size) > target {
			loop {
				let mut tx = self.db.begin().await?;

				let affected = query_file!(
					"src/storage/statements/delete_oldest_batch.sql",
					SHRINK_BATCH,
				)
				.execute(&mut *tx)
				.await?
				.rows_affected();

				if affected == 0 {
					tx.rollback().await?;
					break;
				}

				let counts =
					query_file_as!(PageCounts, "src/storage/statements/db_size.sql")
						.fetch_one(&mut *tx)
						.await?;

				report.deleted = report.deleted.saturating_add(affected);
				tx.commit().await?;

				if counts.live(self.page_size) <= target {
					break;
				}
			}
		}

		let after_delete = self.size().await?;
		if after_delete.physical(self.page_size) > target {
			match self.vacuum().await {
				Ok(()) => report.vacuumed = true,
				Err(e) => {
					warn("failed to vacuum database after trim", attrs!(ERR_KEY => e));
				}
			}
		}

		report.after = self.size().await?.physical(self.page_size);
		Ok(report)
	}

	async fn vacuum(&self) -> Result<()> {
		let mut conn = self.db.acquire().await?;

		query_file!("src/storage/statements/set_auto_vacuum_none.sql")
			.execute(&mut *conn)
			.await?;

		query_file!("src/storage/statements/vacuum.sql")
			.execute(&mut *conn)
			.await?;

		Ok(())
	}

	fn validate_size(cap: u64, target: u64) -> Result<()> {
		if cap <= target {
			return Err(Error::Config(format!(
				"DB_CAP ({cap}) must be greater than DB_TARGET ({target})"
			)));
		}
		Ok(())
	}
}

#[cfg(test)]
mod tests {
	use std::time::UNIX_EPOCH;

	use sqlx::sqlite::{
		SqliteAutoVacuum, SqliteConnectOptions, SqliteJournalMode,
		SqlitePoolOptions,
	};

	use super::*;

	async fn test_storage() -> Storage {
		test_storage_with(SqliteAutoVacuum::None).await
	}

	async fn test_storage_with(auto_vacuum: SqliteAutoVacuum) -> Storage {
		let db = SqlitePoolOptions::new()
			.min_connections(1)
			.max_connections(1)
			.connect_with(
				SqliteConnectOptions::new()
					.in_memory(true)
					.auto_vacuum(auto_vacuum)
					.journal_mode(SqliteJournalMode::Wal),
			)
			.await
			.expect("open test database");
		sqlx::migrate!("src/storage/migrations")
			.run(&db)
			.await
			.expect("run migrations");

		let page_size = query_file_scalar!("src/storage/statements/page_size.sql")
			.fetch_one(&db)
			.await
			.expect("read page size");
		let page_size = u64::try_from(page_size).expect("valid page size");

		Storage { db, page_size }
	}

	async fn insert_pastes(storage: &Storage, count: i64) {
		query_file!(
			"src/storage/statements/seed_test_pastes.sql",
			count,
			vec![b'x'; 100],
		)
		.execute(&storage.db)
		.await
		.expect("insert pastes");
	}

	#[test]
	fn cap_must_exceed_target() {
		assert!(Storage::validate_size(2, 1).is_ok());
		assert!(Storage::validate_size(1, 1).is_err());
		assert!(Storage::validate_size(1, 2).is_err());
	}

	#[test]
	fn page_counts_arithmetic() {
		let counts = PageCounts {
			page_count:     10,
			freelist_count: 4,
		};
		assert_eq!(counts.physical(4096), 40_960);
		assert_eq!(counts.live(4096), 24_576);

		let invalid = PageCounts {
			page_count:     -1,
			freelist_count: 0,
		};
		assert_eq!(invalid.physical(4096), 0);
		assert_eq!(invalid.live(4096), 0);
	}

	#[tokio::test]
	async fn missing_routes_return_none() {
		let storage = test_storage().await;
		let slug = Slug::from_validated("missing".into());

		assert!(storage.delete(&slug).await.expect("delete").is_none());
		assert!(
			storage
				.get_brief(Path::new("missing"))
				.await
				.expect("get brief")
				.is_none()
		);
	}

	#[tokio::test]
	async fn huge_retention_does_not_purge() {
		let storage = test_storage().await;
		insert_pastes(&storage, 3).await;

		let purged =
			storage.purge_expired(Duration::MAX).await.expect("purge expired");
		assert!(purged.is_empty());
	}

	#[tokio::test]
	async fn below_cap_does_nothing() {
		let storage = test_storage().await;
		insert_pastes(&storage, 1).await;

		let page = storage.page_size;
		let report = storage
			.trim_to_spec(page * 10_000, page * 5_000)
			.await
			.expect("trim database");

		assert_eq!(report.deleted, 0);
		assert!(!report.vacuumed);
		assert_eq!(report.before, report.after);
	}

	#[tokio::test]
	async fn trims_oldest_first_then_vacuums() {
		let storage = test_storage().await;
		storage
			.sync_reserved_routes(std::iter::once(Path::new("reserved")))
			.await
			.expect("sync reserved routes");
		insert_pastes(&storage, 600).await;

		let page = storage.page_size;
		let before = storage.size().await.expect("read database size");
		let cap = before.physical(page);
		let target = before.live(page) * 9 / 10;
		assert!(cap > target);

		let report =
			storage.trim_to_spec(cap, target).await.expect("trim database");

		assert_eq!(report.deleted, 512);
		assert!(report.vacuumed);
		assert!(report.after < report.before);
		assert!(report.after <= target);

		let reserved = Slug::from_validated("reserved".into());
		assert!(storage.exists(&reserved).await.expect("check reserved route"));

		let pastes = storage
			.get_created_between(
				UNIX_EPOCH,
				UNIX_EPOCH + Duration::from_secs(1_000_000),
			)
			.await
			.expect("read pastes");
		assert_eq!(pastes.len(), 88);
		let oldest =
			pastes.iter().map(|brief| brief.created).min().expect("oldest paste");
		assert_eq!(oldest, UNIX_EPOCH + Duration::from_secs(513));

		let mode: i64 =
			query_file_scalar!("src/storage/statements/auto_vacuum.sql")
				.fetch_one(&storage.db)
				.await
				.expect("read auto_vacuum");
		assert_eq!(mode, 0);
	}

	#[tokio::test]
	async fn created_between_is_half_open() {
		let storage = test_storage().await;
		insert_pastes(&storage, 10).await;

		let from = UNIX_EPOCH + Duration::from_secs(3);
		let to = UNIX_EPOCH + Duration::from_secs(7);
		let pastes =
			storage.get_created_between(from, to).await.expect("read pastes");

		assert_eq!(pastes.len(), 4);
		let oldest =
			pastes.iter().map(|brief| brief.created).min().expect("oldest paste");
		let newest =
			pastes.iter().map(|brief| brief.created).max().expect("newest paste");
		assert_eq!(oldest, from);
		assert_eq!(newest, UNIX_EPOCH + Duration::from_secs(6));
	}

	#[tokio::test]
	async fn vacuums_without_deleting_when_live_fits() {
		let storage = test_storage().await;
		insert_pastes(&storage, 600).await;

		storage
			.purge_expired(Duration::from_secs(1))
			.await
			.expect("purge expired pastes");

		let page = storage.page_size;
		let before = storage.size().await.expect("read database size");
		let cap = before.physical(page);
		let target = before.live(page) * 2;
		assert!(cap > target);

		let report =
			storage.trim_to_spec(cap, target).await.expect("trim database");

		assert_eq!(report.deleted, 0);
		assert!(report.vacuumed);
		assert!(report.after < report.before);
		assert!(report.after <= target);
	}

	#[tokio::test]
	async fn vacuum_converts_incremental_databases() {
		let storage = test_storage_with(SqliteAutoVacuum::Incremental).await;

		let before_mode: i64 =
			query_file_scalar!("src/storage/statements/auto_vacuum.sql")
				.fetch_one(&storage.db)
				.await
				.expect("read auto_vacuum");
		assert_eq!(before_mode, 2);

		insert_pastes(&storage, 600).await;

		let page = storage.page_size;
		let before = storage.size().await.expect("read database size");
		let cap = before.physical(page);
		let target = before.live(page) * 9 / 10;

		let report =
			storage.trim_to_spec(cap, target).await.expect("trim database");
		assert!(report.vacuumed);

		let after_mode: i64 =
			query_file_scalar!("src/storage/statements/auto_vacuum.sql")
				.fetch_one(&storage.db)
				.await
				.expect("read auto_vacuum");
		assert_eq!(after_mode, 0);
	}
}
