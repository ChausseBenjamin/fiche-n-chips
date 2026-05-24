use actix_web::{HttpRequest, HttpResponse};
use rslog::{attrs, error, src};

use super::client::Client;
use crate::config::logging::ERR_KEY;
use crate::storage::model::{CompressedPaste, Redirect, RowEntry};
use crate::util::decompress;

pub(super) fn handle_entry(req: &HttpRequest, row: RowEntry) -> HttpResponse {
	match row {
		RowEntry::Reserved(res) => {
			error(
				"unexpected query to a reserved page",
				attrs!("path" => res ),
			);
			// TODO: tell the user it's a db sync issue so he may contact me with more
			// specifics if needed.
			error("static web server got bypassed", src!());
			HttpResponse::InternalServerError().finish()
		}
		RowEntry::Redirect(redir) => handle_redirect(&redir),
		RowEntry::Paste(paste) => handle_paste(req, paste),
	}
}

fn handle_redirect(redir: &Redirect) -> HttpResponse {
	HttpResponse::Found()
		.insert_header(("Location", redir.destination()))
		.finish()
}

fn handle_paste(req: &HttpRequest, paste: CompressedPaste) -> HttpResponse {
	let client = Client::from(req);
	let path = paste.slug();
	let comp_data = paste.into_content();
	let mut resp = HttpResponse::Ok();
	if client.accepts_compression() {
		resp.insert_header(("Content-Encoding", "br"));
		resp.body(comp_data)
	} else {
		match decompress(comp_data) {
			Ok(v) => resp.body(v),
			Err(err) => {
				// TODO: tell the user the paste decompression failed so I may
				// investigate if contacted.
				error(
					"failed to decompress user data",
					attrs!(
									ERR_KEY => err,
									"path" => path,
					),
				);
				HttpResponse::InternalServerError().finish()
			}
		}
	}
}
