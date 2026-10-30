use actix_web::http::header::{self, HeaderValue};
use actix_web::{HttpRequest, HttpResponse};
use bytes::Bytes;
use rslog::{attrs, debug, error};

use super::client::{Client, ResponseFmt as RespFmt};
use crate::config::logging::{COMPRESSED_KEY, DEST_KEY, ERR_KEY, SLUG_KEY};
use crate::domain::{
	Brotli, ContentPacked, EntryPacked, Error as DomainError, RecordPacked, Slug,
};
use crate::ui;
use crate::util::sorry;

enum ServeFault {
	Reserved { slug: String },
	Decompress { slug: String, source: DomainError },
	InvalidLocation { slug: String },
}

impl ServeFault {
	fn respond(self, ansi_resp: bool) -> HttpResponse {
		match self {
			Self::Reserved { slug } => {
				error(
					"unexpected query to a reserved page",
					attrs!(SLUG_KEY => slug),
				);
			}
			Self::Decompress { slug, source } => {
				error(
					"failed to decompress stored entry",
					attrs!(SLUG_KEY => slug, ERR_KEY => source),
				);
			}
			Self::InvalidLocation { slug } => {
				error(
					"stored redirect destination is not a valid header value",
					attrs!(SLUG_KEY => slug),
				);
			}
		}

		HttpResponse::InternalServerError().body(ui::format!(
			ansi_resp,
			"An error occured while trying to serve this entry.\n{}",
			sorry()
		))
	}
}

pub(super) fn handle_entry(
	req: &HttpRequest,
	record: EntryPacked,
) -> HttpResponse {
	let client = Client::from(req);
	let ansi_resp = matches!(client.prefered_format(), RespFmt::Ansi);
	let EntryPacked {
		slug,
		entry,
		created,
	} = record;

	let result = match entry {
		EntryPacked::Reserved => Err(ServeFault::Reserved {
			slug: slug.to_string(),
		}),
		EntryPacked::Served(ContentPacked::Redirect(data)) => {
			handle_redirect(slug, data)
		}
		EntryPacked::Served(ContentPacked::Paste(data)) => {
			handle_paste(&client, slug, data)
		}
	};

	match result {
		Ok(resp) => resp,
		Err(fault) => fault.respond(ansi_resp),
	}
}

fn handle_redirect(
	slug: Slug,
	entry: Brotli,
) -> Result<HttpResponse, ServeFault> {
	let dest = entry.decompress().map_err(|source| ServeFault::Decompress {
		slug: slug.to_string(),
		source,
	})?;

	let dest = HeaderValue::from_bytes(dest.as_ref()).map_err(|_| {
		ServeFault::InvalidLocation {
			slug: slug.to_string(),
		}
	})?;

	let location = String::from_utf8_lossy(dest.as_bytes()).into_owned();
	debug("serving redirect", attrs!(DEST_KEY => location));
	Ok(HttpResponse::Found().insert_header((header::LOCATION, dest)).finish())
}

fn handle_paste(
	client: &Client,
	slug: Slug,
	paste: Brotli,
) -> Result<HttpResponse, ServeFault> {
	let compressed = client.accepts_compression();
	debug(
		"serving paste",
		attrs!(SLUG_KEY => slug.to_string(), COMPRESSED_KEY => compressed),
	);

	let mut resp = HttpResponse::Ok();
	resp.insert_header((header::CONTENT_TYPE, "text/plain; charset=utf-8"));
	resp.insert_header(("X-Content-Type-Options", "nosniff"));
	resp.insert_header((header::VARY, "Accept-Encoding"));

	if compressed {
		resp.insert_header((header::CONTENT_ENCODING, "br"));
		return Ok(resp.body(Bytes::from(paste.0.into_vec())));
	}

	let plain = paste.decompress().map_err(|source| ServeFault::Decompress {
		slug: slug.to_string(),
		source,
	})?;
	Ok(resp.body(Bytes::copy_from_slice(plain.as_ref())))
}
