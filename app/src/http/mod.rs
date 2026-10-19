mod api;
mod client;
mod service; // Issued by the service
mod user; // Issued by users

use std::io;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, PoisonError};
use std::time::Duration;

use actix_web::http::Method;
use actix_web::{web, App, HttpRequest, HttpResponse, HttpServer};
use api::{api_delete, api_post};
use client::ResponseFmt as RespFmt;
use crate::config::logging::{METHOD_KEY, PATH_KEY};
use rslog::{attrs, debug};
use service::{handle_404, handle_static};
use tokio_util::sync::CancellationToken;
use user::handle_entry;

use crate::anonymizer::Anonimizer;
use crate::config::Config;
use crate::http::client::Client;
use crate::limiter::{RateLimitStatus, RateLimiter};
use crate::reserved::{ReservedPath as ResPath, ReservedPathProvider, WebFile};
use crate::storage::model::RowEntry;
use crate::storage::{self, Storage};
use crate::ui;
use crate::util::{normalize_http, sorry, SlugGenerator, EMPHASIS, IDENT};

const NOT_FOUND_PATH: &str = "/404";

pub struct Server {
	cfg:          Arc<Config>,
	db:           Arc<Storage>,
	reserved:     Arc<ReservedPathProvider>,
	slugs:        Arc<SlugGenerator>,
	anonimizer:   Arc<Anonimizer>,
	rate_limiter: Arc<Mutex<RateLimiter>>,
}

impl Server {
	pub fn new(
		cfg: Arc<Config>,
		db: Arc<Storage>,
		reserved: Arc<ReservedPathProvider>,
		slugs: Arc<SlugGenerator>,
		anonimizer: Arc<Anonimizer>,
		rate_limiter: Arc<Mutex<RateLimiter>>,
	) -> Self {
		Self {
			cfg,
			db,
			reserved,
			slugs,
			anonimizer,
			rate_limiter,
		}
	}

	pub async fn listen(
		self: Arc<Self>,
		shutdown: CancellationToken,
	) -> io::Result<()> {
		let addr = self.cfg.server.listen_addr.clone();
		let port = self.cfg.server.http_port;
		let timeout = self.cfg.management.shutdown_timeout;
		HttpServer::new(move || {
			App::new()
				.app_data(web::Data::from(self.clone()))
				.default_service(web::to(Server::handler))
		})
		.bind((addr.as_ref(), port))?
		.shutdown_timeout(timeout.as_secs())
		.shutdown_signal(shutdown.cancelled_owned())
		.run()
		.await
	}

	pub async fn handler(
		req: HttpRequest,
		server: web::Data<Self>,
		body: web::Bytes,
	) -> HttpResponse {
		server.dispatch(req, body).await
	}

	async fn dispatch(&self, req: HttpRequest, body: web::Bytes) -> HttpResponse {
		let path = normalize_http(Path::new(req.uri().path()));
		let slug_path = path.strip_prefix("/").unwrap_or(&path).to_path_buf();

		debug(
			"dispatching request",
		attrs!(
			METHOD_KEY => req.method().to_string(),
			PATH_KEY => path.display().to_string()),
		);

		match (req.method(), self.reserved.get(&path)) {
			(&Method::GET, Some(ResPath::File(file))) => {
				debug(
					"matched reserved file",
					attrs!(PATH_KEY => path.display().to_string()),
				);
				handle_static(&req, file)
			}
			(_, Some(ResPath::File(_))) => HttpResponse::MethodNotAllowed()
				.insert_header(("Allow", "GET"))
				.finish(),

			(&Method::POST, Some(ResPath::Api(path))) => {
				if let Some(addr) = req.peer_addr() {
					let discrete = match addr.ip().to_canonical() {
						std::net::IpAddr::V4(v4) => self.anonimizer.cloak(&v4.octets()),
						std::net::IpAddr::V6(v6) => self.anonimizer.cloak(&v6.octets()),
					};
					if let RateLimitStatus::Denied { limit, retry_after } = self
						.rate_limiter
						.lock()
						.unwrap_or_else(PoisonError::into_inner)
						.check_and_increment(discrete)
					{
						let client = Client::from(&req);
						let ansi_resp = matches!(client.prefered_format(), RespFmt::Ansi);
						return HttpResponse::TooManyRequests().body(ui::format!(
							ansi_resp,
							"You've reached the limit ({}).\nPlease try again in {}.\n{}",
							ui::string(limit, IDENT),
							ui::string(
								humantime::format_duration(Duration::from_secs(
									retry_after.as_secs()
								)),
								EMPHASIS
							),
							sorry(),
						));
					}
				}

				api_post(
					&req,
					path.clone(),
					body,
					self.slugs.clone(),
					self.db.clone(),
					&self.cfg,
				)
				.await
			}
			(_, Some(ResPath::Api(_))) => HttpResponse::MethodNotAllowed()
				.insert_header(("Allow", "POST"))
				.finish(),

			(_, Some(ResPath::Alerting(p))) => {
				debug(
					"request hit a path reserved by ntfy",
					attrs!(
					PATH_KEY => p.display().to_string(),
					),
				);
				handle_404(&req, self.get_404(), slug_path.as_path())
			}

			(&Method::GET, None) => {
				if let Ok(p) = Self::get_row(self.db.clone(), slug_path.clone()).await {
					return handle_entry(&req, p);
				}

				debug(
					"user tried to reach a non-existent page",
					attrs!(PATH_KEY => format!("{}", path.display())),
				);
				handle_404(&req, self.get_404(), slug_path.as_path())
			}
			(&Method::DELETE, None) => {
				if let Ok(p) = Self::get_row(self.db.clone(), slug_path.clone()).await {
					return api_delete(&req, p, &self.cfg, self.db.clone()).await;
				}

				debug(
					"user tried to delete a non-existent page",
					attrs!(PATH_KEY => format!("{}", path.display())),
				);
				handle_404(&req, self.get_404(), slug_path.as_path())
			}
			(_, None) => {
				let client = Client::from(&req);
				let mut resp = HttpResponse::MethodNotAllowed();
				resp.insert_header(("Allow", "GET, DELETE"));
				resp.body(ui::format!(
					matches!(client.prefered_format(), RespFmt::Ansi),
					"Using a `{}` method isn't allowed when reaching `{}`.\n{}",
					ui::string(req.method(), EMPHASIS),
					ui::string(path.display(), IDENT),
					sorry(),
				))
			}
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
			ResPath::Alerting(_) => None,
		})
	}
}
