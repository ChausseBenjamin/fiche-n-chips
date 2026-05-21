use std::time::{Duration, UNIX_EPOCH};

use turso::{Connection, Statement};

use super::{error::Error, model::Route};
use crate::{
	storage::model::{
		AgnosticRoute, DeletedRoute, HashedPaste, MatchingPaste, RouteKind,
	},
	util::from_unix_timestamp,
};

#[derive(Clone)]
pub struct Statements(InnerStatements);

#[rustfmt::skip]
struct InnerStatements {
	get_from_path:    Statement,
	get_slugs_by_hash: Statement,
	slug_exists:      Statement,
	del_expired:      Statement,
	del_from_path:    Statement,
	insert_paste:     Statement,
	insert_redir:     Statement,
}

impl Statements {
	pub async fn setup(conn: &Connection) -> Result<Self, Error> {
		Ok(Self( InnerStatements{
			get_from_path: conn
				.prepare_cached("SELECT (id, kind, hash, content, created_on) FROM routes WHERE id = ?")
				.await?,
			get_slugs_by_hash: conn
				.prepare_cached("SELECT (id, content) FROM routes WHERE hash = ?")
				.await?,
      slug_exists: conn
          .prepare_cached("SELECT EXISTS(SELECT 1 FROM routes WHERE id = ?)")
          .await?,
			del_expired: conn
				.prepare_cached(
					"DELETE FROM routes WHERE kind != 0 AND created_on < datetime('now', ?) RETURNING id, created_on")
				.await?,
			del_from_path: conn
				.prepare_cached("DELETE FROM routes WHERE id = ? RETURNING id, created_on")
				.await?,
			insert_paste: conn
				.prepare_cached(
					"INSERT INTO routes (id, kind, hash, content) VALUES (?, 2, ?, ?)")
				.await?,
			insert_redir: conn
				.prepare_cached(
					"INSERT INTO routes (id, kind, hash, content) VALUES (?, 1, NULL, ?)")
				.await?,
      }))
	}

	pub async fn get_from_path(&mut self, path: String) -> Result<Route, Error> {
		let row = self.0.get_from_path.query_row([path]).await?;
		Ok(
			AgnosticRoute {
				id: row.get(0)?,
				kind: (row.get::<i64>(1)?).try_into()?,
				hash: row.get(2)?,
				content: row.get(3)?,
				created: from_unix_timestamp(row.get::<i64>(4)?),
			}
			.into(),
		)
	}

	pub async fn get_slugs_by_hash(
		&mut self,
		hash: u64,
	) -> Result<Vec<MatchingPaste>, Error> {
		let mut rows = self.0.get_slug_by_hash.query([hash as i64]).await?;
		let mut results: Vec<MatchingPaste> = Vec::new();
		while let Some(row) = rows.next().await? {
			results.push(MatchingPaste {
				slug: row.get(0)?,
				content: (row.get::<Option<String>>(1)?).unwrap_or_default(),
			});
		}
		Ok(results)
	}

	pub async fn slug_exists(&mut self, path: String) -> Result<bool, Error> {
		let row = self.0.slug_exists.query_row([path]).await?;
		let exists: u64 = row.get::<i64>(0)? as u64;
		exists == 1
	}

	pub async fn del_expired(
		&mut self,
		range: Duration,
	) -> Result<Vec<DeletedRoute>, Error> {
		let mut rows = self.0.del_expired.query(()).await?;
		let mut results: Vec<DeletedRoute> = Vec::new();
		while let Some(row) = rows.next().await? {
			results.push(DeletedRoute {
				id: row.get(0)?,
				created: from_unix_timestamp(row.get::<i64>(1)?),
			});
		}
		Ok(results)
	}

	pub async fn del_from_path(
		&mut self,
		path: String,
	) -> Result<DeletedRoute, Error> {
		let row = self.0.del_from_path.query_row([path]).await?;
		Ok(DeletedRoute {
			id: row.get(0)?,
			created: from_unix_timestamp(row.get::<i64>(1)?),
		})
	}

	pub async fn insert_paste(
		&mut self,
		entry: HashedPaste,
	) -> Result<(), Error> {
		todo!()
	}

	pub async fn insert_redir(
		&mut self,
		path: String,
		content: Vec<u8>,
	) -> Result<(), Error> {
		todo!()
	}
}
