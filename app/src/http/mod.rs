mod api;
mod client;
mod service; // Issued by the service
mod user; // Issued by users

use std::path::{Path, PathBuf};
use std::sync::Arc;

use actix_web::http::Method;
use actix_web::{web, HttpRequest, HttpResponse};
use api::{api_delete, api_post};
use rslog::{attrs, debug};
use service::{handle_404, handle_static};
use user::handle_entry;

use crate::config::Config;
use crate::reserved::{ReservedPath as ResPath, ReservedPathProvider, WebFile};
use crate::storage::model::RowEntry;
use crate::storage::{self, Storage};
use crate::util::normalize_http;

const NOT_FOUND_PATH: &str = "/404";

pub struct Server {
	cfg:      Arc<Config>,
	db:       Arc<Storage>,
	reserved: Arc<ReservedPathProvider>,
}

impl Server {
	pub fn new(
		cfg: Arc<Config>,
		db: Arc<Storage>,
		reserved: Arc<ReservedPathProvider>,
	) -> Self {
		Self { cfg, db, reserved }
	}

	async fn dispatch(&self, req: HttpRequest) -> HttpResponse {
		let path = normalize_http(Path::new(req.uri().path()));
		let slug_path = path.strip_prefix("/").unwrap_or(&path).to_path_buf();

		match (req.method(), self.reserved.get(&path)) {
			(&Method::GET, Some(ResPath::File(file))) => handle_static(&req, file),
			(_, Some(ResPath::File(_))) => HttpResponse::MethodNotAllowed()
				.insert_header(("Allow", "GET"))
				.finish(),

			(&Method::POST, Some(ResPath::Api(path))) => {
				api_post(req, path.clone()).await
			}
			(_, Some(ResPath::Api(_))) => HttpResponse::MethodNotAllowed()
				.insert_header(("Allow", "POST"))
				.finish(),

			(&Method::GET, None) => {
				if let Ok(p) = Self::get_row(self.db.clone(), slug_path.clone()).await {
					return handle_entry(&req, p);
				}

				debug(
					"user tried to reach a non-existent page",
					attrs!("path" => format!("{}", path.display())),
				);
				handle_404(&req, self.get_404(), slug_path.as_path())
			}
			(&Method::DELETE, None) => {
				if let Ok(p) = Self::get_row(self.db.clone(), slug_path.clone()).await {
					return api_delete(req, p).await;
				}

				debug(
					"user tried to delete a non-existent page",
					attrs!("path" => format!("{}", path.display())),
				);
				handle_404(&req, self.get_404(), slug_path.as_path())
			}
			(_, None) => HttpResponse::MethodNotAllowed()
				.insert_header(("Allow", "GET, DELETE"))
				.finish(),
		}
	}

	async fn get_row(
		db: Arc<Storage>,
		path: PathBuf,
	) -> Result<RowEntry, storage::Error> {
		db.queries.get_from_path(path.to_string_lossy().to_string()).await
	}

	fn get_404(&self) -> Option<&WebFile<'static>> {
		self.reserved.get(Path::new(NOT_FOUND_PATH)).and_then(|r| match r {
			ResPath::File(f) => Some(f),
			ResPath::Api(_) => None,
		})
	}

	pub async fn handler(
		req: HttpRequest,
		server: web::Data<Self>,
	) -> HttpResponse {
		server.dispatch(req).await
	}
}
