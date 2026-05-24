use std::borrow::Cow;
use std::path::Path;

use actix_web::{HttpRequest, HttpResponse};
use rslog::{attrs, error};

use super::client::{Client, ResponseFmt as RespFmt};
use crate::config::logging::ERR_KEY;
use crate::reserved::{FileContents, WebFile};
use crate::ui::{self, UiString};
use crate::util::{decompress, sorry};

pub(super) fn handle_static(
	req: &HttpRequest,
	endpoint: &'_ WebFile<'static>,
) -> HttpResponse {
	let mut resp = HttpResponse::Ok();
	let client = Client::from(req);
	let content: Cow<'_, [u8]> = match endpoint.content() {
		FileContents::Raw(cow) => cow.clone(),
		FileContents::Brotli(cow) => {
			if client.accepts_compression() {
				resp.insert_header(("Content-Encoding", "br"));
				cow.clone()
			} else {
				match decompress(cow.to_vec()) {
					Ok(s) => Cow::Owned(s.into_bytes()),
					Err(err) => {
						error(
							"static file decompression failed",
							attrs!(
                "path" => format!("{}",endpoint.path().display()),
                ERR_KEY => err),
						);
						return HttpResponse::InternalServerError().finish();
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
	macro_rules! tmplt {
		() => {
			"You requested '{}' but it doesn't seem to exist.\n{}"
		};
	}

	let mut resp = HttpResponse::NotFound();
	let client = Client::from(req);

	let origin_txt: UiString =
		ui::string(origin.display(), &[ui::BOLD, ui::YELLOW_FG]);

	match client.prefered_format() {
		RespFmt::Ansi => resp.body(format!(tmplt!(), origin_txt, sorry())),
		RespFmt::Plain => resp.body(crate::plain!(tmplt!(), origin_txt, sorry())),
		RespFmt::Html => {
			if let Some(page) = endpoint {
				handle_static(req, page)
			} else {
				resp.finish()
			}
		}
	}
}
