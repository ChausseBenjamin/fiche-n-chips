use actix_web::HttpRequest;

pub struct Client(ClientInner);

struct ClientInner {
	accepts_compression: bool,
	preferred_format:    ResponseFmt,
}

pub enum ResponseFmt {
	Html,
	Ansi,
	Plain,
}

impl Client {
	/// Determines wether or not the server must bother decompressing
	/// data or just send it as-is to the client with a header
	pub fn accepts_compression(&self) -> bool { self.0.accepts_compression }

	pub fn prefered_format(&self) -> &ResponseFmt { &self.0.preferred_format }
}

impl From<&HttpRequest> for Client {
	fn from(req: &HttpRequest) -> Self {
		// Browsers tend to be specific about what they accept and explicitly state
		// `text/html` or other. As curl remains global and uses `*/*` by default,
		// this is a quick and dirty fix.
		let accepts_html = req
			.headers()
			.get("Accept")
			.and_then(|v| v.to_str().ok())
			.is_some_and(|v| v.contains("html"));
		let mut format =
			if accepts_html { ResponseFmt::Html } else { ResponseFmt::Ansi };
		// Allow users to explicitly disable ANSI colors by adding a custom header
		// with basically anything (so `y`,`yes`,`1`,`true`,etc...
		if let Some(val) =
			req.headers().get("X-Disable-Colors").and_then(|v| v.to_str().ok())
		{
			let trimmed = val.trim();
			if !trimmed.is_empty() && trimmed != "false" {
				format = ResponseFmt::Plain;
			}
		}
		let accepts_compression = req
			.headers()
			.get("accept-encoding")
			.and_then(|v| v.to_str().ok())
			.is_some_and(|v| v.split(',').any(|enc| enc.trim() == "br"));
		Self(ClientInner {
			accepts_compression,
			preferred_format: format,
		})
	}
}
