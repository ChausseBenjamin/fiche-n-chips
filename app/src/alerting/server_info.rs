use serde::Serialize;

use super::emitter::AlertMsg;

#[derive(Serialize)]
#[serde(tag = "state", rename_all = "lowercase")]
pub enum ServerInfo {
	Startup,
	Shutdown,
}

impl AlertMsg for ServerInfo {
	const TEMPLATE: &'static str = "fnp-server";

	fn topic(&self) -> &str { "status" }
}
