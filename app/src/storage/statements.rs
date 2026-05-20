use turso::{Connection, Statement};

use super::error::Error;

#[rustfmt::skip]
#[derive(Clone)]
pub struct Statements {
	pub get_from_path:    Statement,
	pub get_slug_by_hash: Statement,
	pub del_expired:      Statement,
	pub del_from_path:    Statement,
	pub del_from_hash:    Statement,
	pub insert_paste:     Statement,
	pub insert_redir:     Statement,
}

impl Statements {
	pub async fn setup(conn: &Connection) -> Result<Self, Error> {
		Ok(Self {
			get_from_path: conn
				.prepare_cached("SELECT * FROM routes WHERE id = ?")
				.await?,
			get_slug_by_hash: conn
				.prepare_cached("SELECT id FROM routes WHERE hash = ?")
				.await?,
			del_expired: conn
				.prepare_cached(
					"DELETE FROM routes WHERE kind != 0 AND created_on < datetime('now', ?)",
				)
				.await?,
			del_from_path: conn
				.prepare_cached("DELETE FROM routes WHERE id = ?")
				.await?,
			del_from_hash: conn
				.prepare_cached("DELETE FROM routes WHERE hash = ?")
				.await?,
			insert_paste: conn
				.prepare_cached(
					"INSERT INTO routes (id, kind, hash, content) VALUES (?, 2, ?, ?)",
				)
				.await?,
			insert_redir: conn
				.prepare_cached(
					"INSERT INTO routes (id, kind, hash, content) VALUES (?, 1, NULL, ?)",
				)
				.await?,
		})
	}
}
