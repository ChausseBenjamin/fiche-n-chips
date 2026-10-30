use std::path::Path;

use super::ReservedPath;

pub const SHORT_URL_PATH: &str = "/api/shorten";
pub const HEALTHCHECK_PATH: &str = "/api/health";

pub fn api_paths() -> Vec<ReservedPath> {
	vec![
		ReservedPath::Api(Path::new(SHORT_URL_PATH)),
		ReservedPath::Api(Path::new(HEALTHCHECK_PATH)),
	]
}
