use std::time::{Duration, UNIX_EPOCH};

use turso::{Connection, Statement};

use super::{error::Error, model::Route};
use crate::{
	storage::model::{
		AgnosticRoute, CompPaste, CompRedirect, DeletedRoute, MatchingPaste,
		RouteKind,
	},
	util::from_unix_timestamp,
};

#[derive(Clone)]
pub struct Statements(InnerStatements);

#[rustfmt::skip]
struct InnerStatements {
	get_from_path:     Statement,
	get_slugs_by_hash: Statement,
	slug_exists:       Statement,
	del_expired:       Statement,
	del_from_path:     Statement,
	insert_route:      Statement,
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
			insert_route: conn
				.prepare_cached(
					"INSERT INTO routes (id, kind, hash, content) VALUES (?, ?, ?, ?)")
				.await?,
      }))
	}

	pub async fn get_from_path(&self, path: String) -> Result<Route, Error> {
		let mut cmd = self.0.get_from_path.clone();
		let row = cmd.query_row([path]).await?;
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
		&self,
		hash: u64,
	) -> Result<Vec<MatchingPaste>, Error> {
		let mut cmd = self.0.get_slugs_by_hash.clone();
		let mut rows = cmd.query([hash as i64]).await?;
		let mut results: Vec<MatchingPaste> = Vec::new();
		while let Some(row) = rows.next().await? {
			results.push(MatchingPaste {
				slug: row.get(0)?,
				content: (row.get::<Option<Vec<u8>>>(1)?).unwrap_or_default(),
			});
		}
		Ok(results)
	}

	pub async fn slug_exists(&self, path: &str) -> Result<bool, Error> {
		let mut cmd = self.0.slug_exists.clone();
		let row = cmd.query_row([path]).await?;
		let exists: u64 = row.get::<i64>(0)? as u64;
		Ok(exists == 1)
	}

	pub async fn del_expired(
		&self,
		range: Duration,
	) -> Result<Vec<DeletedRoute>, Error> {
		let mut cmd = self.0.del_expired.clone();
		let mut rows = cmd.query(()).await?;
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
		&self,
		path: String,
	) -> Result<DeletedRoute, Error> {
		let mut cmd = self.0.del_from_path.clone();
		let row = cmd.query_row([path]).await?;
		Ok(DeletedRoute {
			id: row.get(0)?,
			created: from_unix_timestamp(row.get::<i64>(1)?),
		})
	}

	async fn insert_route(&self, route: AgnosticRoute) -> Result<(), Error> {
		let id = turso::Value::from(route.id);
		let kind = turso::Value::from(route.kind as u8);
		let hash = turso::Value::from(route.hash);
		let content = turso::Value::from(route.content);

		let mut cmd = self.0.insert_route.clone();
		cmd.execute([id, kind, hash, content]).await?;
		Ok(())
	}

	pub async fn insert_paste(&self, paste: CompPaste) -> Result<(), Error> {
		let route: AgnosticRoute = Route::from(paste).into();
		self.insert_route(route).await
	}

	pub async fn insert_redirect(
		&self,
		redirect: CompRedirect,
	) -> Result<(), Error> {
		let route: AgnosticRoute = Route::from(redirect).into();
		self.insert_route(route).await
	}
}
