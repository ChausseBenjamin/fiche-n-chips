use std::path::Path;
use std::sync::Arc;

use actix_web::{HttpRequest, HttpResponse, web};
use rslog::{attrs, debug, error};
use serde::{Deserialize, Serialize};

use super::super::Client;
use crate::config::Config;
use crate::config::logging::{ERR_KEY, PATH_KEY};
use crate::http::client::ResponseFmt as RespFmt;
use crate::storage::Storage;
use crate::storage::model::{RedirectInput, Storable};
use crate::ui;
use crate::util::{CONTACT, IDENT, SlugGenerator, sorry, success};

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
	endpoint: &Path,
	body: web::Bytes,
	slugs: Arc<SlugGenerator>,
	db: Arc<Storage>,
	cfg: &Config,
) -> HttpResponse {
	let ansi_resp: bool = match Client::from(req).prefered_format() {
		RespFmt::Ansi => true,
		RespFmt::Plain | RespFmt::Html => false,
	};

	#[allow(clippy::single_match_else)]
	match endpoint.to_string_lossy().as_ref() {
		"/api/shorten" => handle_shorten(body, slugs, db, cfg, ansi_resp).await,
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
	slugs: Arc<SlugGenerator>,
	db: Arc<Storage>,
	cfg: &Config,
	ansi_resp: bool,
) -> HttpResponse {
	let req: ShortenReq = match serde_json::from_slice::<ShortenReq>(&body) {
		Ok(json) => json,
		Err(e) => {
			debug(
				"failed to parse user request for a short url",
				attrs!(ERR_KEY => e),
			);
			return HttpResponse::BadRequest().body(ui::format!(
				ansi_resp,
				"Invalid request, format must be valid json as follows:\n\t`{}`\n{}",
				short_req_demo(),
				sorry(),
			));
		}
	};

	#[allow(clippy::cast_possible_truncation)]
	let max_url = cfg.redirect.max_raw.get() as usize;
	if req.url.len() > max_url {
		return HttpResponse::BadRequest().body(ui::format!(
			ansi_resp,
			"The provided URL length of {} exceeds the maximum allowed ({}).\nPlease use a shorter URL and try again.\n{}",
			ui::string(req.url.len(), IDENT),
			max_url,
			sorry(),
		));
	}
	if let Some(ref slug) = req.slug {
		let max_slug = cfg.redirect.max_short as usize;
		if slug.len() > max_slug {
			return HttpResponse::BadRequest().body(ui::format!(
				ansi_resp,
				"The provided slug `{}` exceeds the maximum allowed length of {}.\nPlease use a shorter slug and try again.\n{}",
				ui::string(slug, IDENT),
				max_slug,
				sorry(),
			));
		}
		if db.queries.slug_exists(slug).await.unwrap_or(false) {
			return HttpResponse::Conflict().body(ui::format!(
				ansi_resp,
				"Cannot use `{}` as it's already in use by another redirect.\nPlease choose a different slug and try again.\n{}",
				ui::string(slug, IDENT),
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
				attrs!(ERR_KEY => e),
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
			db.spawn_shrink(cfg.mgmt.db_size.get());
			HttpResponse::Ok().body(ui::format!(ansi_resp,
				"Successfully created a short url for the requested target, {}\nYou may use the url below:\n{}/{}\n",
				success(),
				cfg.server.public_addr.to_string(),
				redirect.id(),
			))
		}
		Err(e) => {
			error(
				"failed to insert properly generated short-url into the db",
				attrs!(ERR_KEY => e),
			);
			HttpResponse::InternalServerError().body(ui::format!(ansi_resp,
				"An internal failure occured while adding your shortened url to the database.\n{}",
				sorry(),
			))
		}
	}
}
