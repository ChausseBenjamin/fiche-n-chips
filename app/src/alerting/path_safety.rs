use std::path::PathBuf;

const RESERVED_NTFY_PATHS: &[&str] = &[
	"/docs",
	"/static",
	"/file",
	"/app",
	"/metrics",
	"/account",
	"/settings",
	"/signup",
	"/login",
	"/v1",
	// Not blocked by ntfy but used as a topic for global (non-slug) messages:
	"/status",
];

pub fn reserved_paths() -> Vec<PathBuf> {
	RESERVED_NTFY_PATHS.iter().map(|p| PathBuf::from(*p)).collect()
}
