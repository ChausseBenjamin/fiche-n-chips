use std::borrow::Cow;
use std::path::Path;

use actix_web::{HttpRequest, HttpResponse};
use rslog::{attrs, debug, error};

use super::client::{Client, ResponseFmt as RespFmt};
use crate::config::logging::{ERR_KEY, MIME_KEY, PATH_KEY};
use crate::reserved::{FileContents, WebFile};
use crate::ui::{self, UiString};
use crate::util::{decompress, sorry, IDENT};

pub(super) fn handle_static(
	req: &HttpRequest,
	endpoint: &'_ WebFile<'static>,
) -> HttpResponse {
	debug(
		"serving static file",
		attrs!(
			PATH_KEY => endpoint.path().display().to_string(),
			MIME_KEY => endpoint.mime().to_owned()),
	);
	let mut resp = HttpResponse::Ok();
	resp.insert_header(("Content-Type", endpoint.mime()));
	resp.insert_header(("X-Content-Type-Options", "nosniff"));
	let client = Client::from(req);
	let content: Cow<'_, [u8]> = match endpoint.content() {
		FileContents::Raw(cow) => cow.clone(),
		FileContents::Brotli(cow) => {
			if client.accepts_compression() {
				resp.insert_header(("Content-Encoding", "br"));
				cow.clone()
			} else {
				match decompress(cow) {
					Ok(s) => Cow::Owned(s.into_bytes()),
					Err(err) => {
						error(
							"static file decompression failed",
							attrs!(
                PATH_KEY => format!("{}",endpoint.path().display()),
                ERR_KEY => err),
						);
						let mut resp = HttpResponse::InternalServerError();
						resp.insert_header(("Content-Type", "text/plain; charset=utf-8"));
						return resp.body(ui::format!(
							matches!(client.prefered_format(), RespFmt::Ansi),
							"You requested `{}` but the server failed to decompress it.\n{}",
							ui::string(endpoint.path().display(), IDENT),
							sorry(),
						));
					}
				}
			}
		}
	};
	resp.body(content.to_vec())
}

pub(super) fn handle_404(
	req: &HttpRequest,
	endpoint: Option<&'_ WebFile<'static>>,
	origin: &Path,
) -> HttpResponse {
	let mut resp = HttpResponse::NotFound();
	resp.insert_header(("Content-Type", "text/plain; charset=utf-8"));
	let client = Client::from(req);

	let origin_txt: UiString = ui::string(origin.display(), IDENT);

	match client.prefered_format() {
		RespFmt::Ansi | RespFmt::Plain => resp.body(ui::format!(
			matches!(client.prefered_format(), RespFmt::Ansi),
			"You requested `{}` but it doesn't seem to exist.\n{}",
			origin_txt,
			sorry(),
		)),
		RespFmt::Html => {
			if let Some(page) = endpoint {
				handle_static(req, page)
			} else {
				resp.finish()
			}
		}
	}
}
