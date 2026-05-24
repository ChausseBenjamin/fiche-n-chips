use std::path::PathBuf;

use super::ReservedPath;

const SHORT_URL_PATH: &str = "/shorten";

pub fn api_paths() -> Vec<ReservedPath> {
	vec![ReservedPath::Api(PathBuf::from(SHORT_URL_PATH))]
}
