use std::path::Path;
use std::sync::Arc;

use actix_web::{HttpRequest, HttpResponse, web};
use rslog::{attrs, debug, error};
use serde::{Deserialize, Serialize};

use super::super::Client;
use crate::config::Config;
use crate::config::logging::{ERR_KEY, PATH_KEY};
use crate::domain::{self, RedirectLimits, Resolved, UploadPending};
use crate::http::client::ResponseFmt as RespFmt;
use crate::storage::{Error as StorageError, Storage};
use crate::ui;
use crate::util::{CONTACT, IDENT, sorry, success};

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
	pub slug: Option<Box<str>>,
}

/// Everything that can go wrong while shortening a URL.
///
/// Kept separate from [`HttpResponse`] so the happy path can use `?` and the
/// status/message mapping lives in exactly one place.
enum ShortenError {
	Malformed(serde_json::Error),
	Rejected(domain::Error),
	Conflict(String),
	Storage(StorageError),
}

impl ShortenError {
	fn respond(self, ansi_resp: bool) -> HttpResponse {
		use domain::Error as E;
		match self {
			Self::Malformed(e) => {
				debug(
					"failed to parse user request for a short url",
					attrs!(ERR_KEY => e),
				);
				HttpResponse::BadRequest().body(ui::format!(
					ansi_resp,
					"Invalid request, format must be valid json as follows:\n\t`{}`\n{}",
					short_req_demo(),
					sorry(),
				))
			}
			Self::Rejected(E::UrlTooLong { max, actual }) => {
				HttpResponse::BadRequest().body(ui::format!(
					ansi_resp,
					"The provided URL length of {} exceeds the maximum allowed ({}).\n\
          Please use a shorter URL and try again.\n{}",
					ui::string(actual, IDENT),
					max,
					sorry(),
				))
			}
			Self::Rejected(E::InvalidSlug { reason }) => HttpResponse::BadRequest()
				.body(ui::format!(
					ansi_resp,
					"The provided slug isn't valid: {}.\n\
          Please choose a different slug and try again.\n{}",
					ui::string(reason, IDENT),
					sorry(),
				)),
			Self::Rejected(e) => {
				error("failed to create redirect upload", attrs!(ERR_KEY => e));
				HttpResponse::InternalServerError().body(ui::format!(
					ansi_resp,
					"A failure occured while trying to create the short url.\n{}",
					sorry(),
				))
			}
			Self::Conflict(slug) => HttpResponse::Conflict().body(ui::format!(
				ansi_resp,
				"Cannot use `{}` as it's already in use by another redirect.\n\
        Please choose a different slug and try again.\n{}",
				ui::string(slug, IDENT),
				sorry(),
			)),
			Self::Storage(e) => {
				error("failed to generate the short url", attrs!(ERR_KEY => e));
				HttpResponse::InternalServerError().body(ui::format!(
					ansi_resp,
					"A failure occured while trying to generate the short url.\n{}",
					sorry(),
				))
			}
		}
	}
}

pub async fn api_post(
	req: &HttpRequest,
	endpoint: &Path,
	body: web::Bytes,
	db: Arc<Storage>,
	cfg: &Config,
) -> HttpResponse {
	let ansi_resp: bool = match Client::from(req).prefered_format() {
		RespFmt::Ansi => true,
		RespFmt::Plain | RespFmt::Html => false,
	};

	#[allow(clippy::single_match_else)]
	match endpoint.to_string_lossy().as_ref() {
		"/api/shorten" => match handle_shorten(body, &db, cfg, ansi_resp).await {
			Ok(resp) => resp,
			Err(e) => e.respond(ansi_resp),
		},
		_ => {
			let mut resp = HttpResponse::InternalServerError();
			error(
				"user reached a supposedly unreachable api endpoint",
				attrs!(PATH_KEY => endpoint.to_string_lossy().to_string()),
			);
			resp.body(ui::format!(
				ansi_resp,
				"You reached an unknown API path, this is a bug.\nIf you can spare the time, could you please report this to <{}> so he may look into the issue?\n{}",
				ui::string(cfg.server.maintainer_email.clone(), CONTACT),
				sorry(),
			))
		}
	}
}

async fn handle_shorten(
	body: web::Bytes,
	db: &Storage,
	cfg: &Config,
	ansi_resp: bool,
) -> Result<HttpResponse, ShortenError> {
	let ShortenReq { url, slug } = serde_json::from_slice::<ShortenReq>(&body)
		.map_err(ShortenError::Malformed)?;

	let limits = RedirectLimits::from(&cfg.redirect);
	let redirect =
		UploadPending::try_redirect(url.into_bytes(), slug.as_deref(), limits)
			.map_err(ShortenError::Rejected)?;

	if let Some(slug) = redirect.requested_slug() {
		if db.exists(slug).await.map_err(ShortenError::Storage)? {
			return Err(ShortenError::Conflict(slug.to_string()));
		}
	}

	let candidates = db
		.potential_duplicates(&redirect)
		.await
		.map_err(ShortenError::Storage)?;
	let redirect =
		match redirect.resolve(candidates).map_err(ShortenError::Rejected)? {
			Resolved::Novel(redirect) => redirect,
			Resolved::Existing(brief) => {
				return Err(ShortenError::Conflict(brief.slug.to_string()));
			}
		};
	let redirect = redirect.prepare().map_err(ShortenError::Rejected)?;
	let brief = db.insert(redirect).await.map_err(|e| match e {
		StorageError::SlugTaken(slug) => ShortenError::Conflict(slug.to_string()),
		other => ShortenError::Storage(other),
	})?;

	Ok(HttpResponse::Ok().body(ui::format!(
		ansi_resp,
		"Successfully created a short url for the requested target, {}\n\
    You may use the url below:\n{}/{}\n",
		success(),
		&cfg.server.public_addr,
		brief.slug,
	)))
}
