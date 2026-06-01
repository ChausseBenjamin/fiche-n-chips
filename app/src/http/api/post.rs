use std::path::PathBuf;
use std::sync::Arc;

use actix_web::{web, HttpRequest, HttpResponse};
use rslog::{attrs, debug, error};
use serde::{Deserialize, Serialize};

use super::super::Client;
use crate::config::logging::ERR_KEY;
use crate::config::Config;
use crate::http::client::ResponseFmt as RespFmt;
use crate::storage::model::{RedirectInput, Storable};
use crate::storage::Storage;
use crate::ui;
use crate::util::{sorry, SlugGenerator};
use turso::Error as DbError;

fn short_req_demo() -> ui::UiString {
	ui::string(
		r#"{ "url": "https://example.com/very/long/url", "slug": "optional_short_path" }"#,
		&[ui::BLUE_BG],
	)
}

#[derive(Serialize, Deserialize)]
pub struct ShortenReq {
	pub url:  String,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub slug: Option<String>,
}

pub async fn api_post(
	req: &HttpRequest,
	endpoint: PathBuf,
	body: web::Bytes,
	slugs: Arc<SlugGenerator>,
	db: Arc<Storage>,
	cfg: &Config,
) -> HttpResponse {
	let ansi_resp: bool = match Client::from(req).prefered_format() {
		RespFmt::Ansi => true,
		RespFmt::Plain | RespFmt::Html => false,
	};
	// NOTE: If more endpoints get made, create standalone functions for each
	#[allow(clippy::single_match_else)]
	match endpoint.to_string_lossy().as_ref() {
		"/api/shorten" => {
			let req: ShortenReq = match serde_json::from_slice::<ShortenReq>(&body) {
				Ok(json) => json,
				Err(e) => {
					debug(
						"failed to parse user request for a short url",
						attrs!(
							ERR_KEY => e,
						),
					);
					return HttpResponse::BadRequest().body(ui::format!(
						ansi_resp,
						"Invalid request, format must be valid json as follows:\n\t`{}`\n{}",
						short_req_demo(),
						sorry(),
					));
				}
			};

			let max_url = cfg.redirect.max_raw.get() as usize;
			if req.url.len() > max_url {
				return HttpResponse::BadRequest().body(ui::format!(
					ansi_resp,
					"The provided URL exceeds the maximum allowed length ({} > {}).\n{}",
					req.url.len(),
					max_url,
					sorry(),
				));
			}
			if let Some(ref slug) = req.slug {
				let max_slug = cfg.redirect.max_short as usize;
				if slug.len() > max_slug {
					return HttpResponse::BadRequest().body(ui::format!(
						ansi_resp,
						"The provided slug exceeds the maximum allowed length ({} > {}).\n{}",
						slug.len(),
						max_slug,
						sorry(),
					));
				}
				if db.queries.slug_exists(slug).await.unwrap_or(false) {
					return HttpResponse::Conflict().body(ui::format!(
						ansi_resp,
						"The slug `{}` is already in use by another redirect.\n{}",
						slug,
						sorry(),
					));
				}
			}

			let redirect = match RedirectInput::new(req.url, req.slug)
				.try_into_compressed_redirect(&db, slugs)
				.await
			{
				Ok(v) => v,
				Err(e) => {
					debug(
						"failed to convert shorten request to a database-ready format",
						attrs!(
							ERR_KEY => e,
						),
					);
					return HttpResponse::InternalServerError().body(ui::format!(
						ansi_resp,
						"A failure occured while trying to generate the short url.\n{}",
						sorry()
					));
				}
			};
			match db.queries.insert(&redirect).await {
				Ok(_) => {
					HttpResponse::Ok().body(ui::format!(ansi_resp,
                "Successfully created a short url for the requested target, {}\nYou may use the url below:\n{}/{}\n",
                ui::string("cheers!", &[ui::GREEN_FG]),
                cfg.server.public_addr.to_string(),
                redirect.id(),
        ))
				}
				Err(e) => match &e {
					crate::storage::Error::DbErr(DbError::Constraint(_)) => {
						HttpResponse::Conflict().body(ui::format!(
							ansi_resp,
							"That slug appears to already be taken — please try a different one.\n{}",
							sorry(),
						))
					}
					_ => {
						error(
							"failed to insert properly generated short-url into the db",
							attrs!(ERR_KEY => e),
						);
						HttpResponse::InternalServerError().body(ui::format!(ansi_resp,
							"An internal failure occured while adding your shortened url to the database.\n{}",
							sorry(),
						))
					}
				},
			}
		}
		_ => {
			let mut resp = HttpResponse::InternalServerError();
			// TODO: log an error internally
			error(
				"user reached a supposedly unreachable api endpoint",
				attrs!(
					"path" => endpoint.to_string_lossy().to_string(),
				),
			);
			resp.body(ui::format!(
				ansi_resp,
				"You reached an unknown API path, this is a bug.\nIf you can spare the time, could you please report this to <{}> so he may look into the issue?\n{}",

        ui::string(cfg.server.maintainer_email.clone(), &[ui::BOLD, ui::UNDERLINE]),
				sorry(),
			))
		}
	}
}
