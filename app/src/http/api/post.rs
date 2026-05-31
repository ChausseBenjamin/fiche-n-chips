use std::path::PathBuf;

use actix_web::body::to_bytes;
use actix_web::{web, HttpRequest, HttpResponse};
use serde::{Deserialize, Serialize};

use super::super::Client;
use crate::config::Config;
use crate::http::client::ResponseFmt as RespFmt;
use crate::ui;
use crate::util::sorry;

#[derive(Serialize, Deserialize)]
pub struct ShortenRequest {
	pub url:  String,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub slug: Option<String>,
}

pub async fn api_post(
	req: &HttpRequest,
	endpoint: PathBuf,
	cfg: &Config,
) -> HttpResponse {
	let ansi_resp: bool = match Client::from(req).prefered_format() {
		RespFmt::Ansi => true,
		RespFmt::Plain | RespFmt::Html => false,
	};
	match endpoint.to_string_lossy().as_ref() {
		"/api/shorten" => todo!(),
		_ => {
			let mut resp = HttpResponse::InternalServerError();
			let bytes = match to_bytes(req.into_body()) {
				Ok(json) => json.into_inner(),
				Err(e) => {
					let mut resp = HttpResponse::BadRequest();
					return resp.body(ui::format!(ansi_resp,
                "Invalid request body, format must be as follows:\n\t`{}`\n{}",
                ui::string(
                  r#"{ "url": "https://example.com/very/long/url", "slug": "optional_short_path" }"#,
                  &[ui::BLUE_BG],
                ),
                sorry(),
              ));
				}
			};
			// TODO: log an error internally
			return resp.body(ui::format!(
				ansi_resp,
				"You reached an unknown API path, this is a bug.\nIf you can spare the time, could you please report this to <{}> so he may look into the issue?\n{}",

        ui::string(cfg.server.maintainer_email.clone(), &[ui::BOLD, ui::UNDERLINE]),
				sorry(),
			));
		}
	}
}
