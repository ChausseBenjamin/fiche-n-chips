use std::path::Path;

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
	// Not blocked by ntfy but will be used as a topic for global alerts
	// which aren't slug-specific (Soon-TM)
	"/status",
];

pub fn reserved_paths() -> Vec<&'static Path> {
	RESERVED_NTFY_PATHS.iter().map(|p| Path::new(*p)).collect()
}
