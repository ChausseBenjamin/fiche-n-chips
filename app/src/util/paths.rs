use std::fs::{metadata, OpenOptions};
use std::io::{Error, ErrorKind};
use std::path::{Path, PathBuf};

const INDEX: &str = "index";

/// Validates and normalizes a file path for file-based configuration outputs.
///
/// The function accepts either an existing file path or a directory path:
/// - If `path` is an existing file, it is verified as writable (append mode)
///   and returned.
/// - If `path` is an existing directory, `filename` is appended and the
///   resulting file path is returned.
/// - If `path` does not exist but its parent directory exists, it is treated as
///   a directory path and `filename` is appended.
/// - Otherwise, an error is returned.
///
/// # Arguments
/// * `path` - File path or directory path
/// * `filename` - Default filename used when treating `path` as a directory
///
/// # Returns
/// A fully resolved `PathBuf` pointing to a file, or an `io::Error` if
/// validation fails.
pub fn valid_filepath<P: AsRef<Path>>(
	path: P,
	filename: &str,
) -> Result<PathBuf, Error> {
	let path = path.as_ref();

	match metadata(path) {
		Ok(meta) if meta.is_file() => {
			OpenOptions::new().append(true).open(path)?;
			Ok(path.to_path_buf())
		}

		Ok(meta) if meta.is_dir() => Ok(path.join(filename)),

		Ok(_) => Err(Error::new(
			ErrorKind::InvalidInput,
			format!(
				"Path '{}' is neither a file nor a directory",
				path.display()
			),
		)),

		Err(e) if e.kind() == ErrorKind::NotFound => {
			if let Some(parent) = path.parent() {
				if parent.exists() && metadata(parent)?.is_dir() {
					return Ok(parent.join(filename));
				}
			}

			Err(Error::new(
				ErrorKind::NotFound,
				format!("Parent directory for '{}' does not exist", path.display()),
			))
		}

		Err(e) => Err(e),
	}
}

/// Generates a path that makes sense in the context of http
/// 1. Ignore file extensions:
///    - `/hello.txt` ~ `/hello.html` => `/hello`
/// 2. Ignore trailing slashes:
///    - `/docs/` => `/docs`
/// 3. Treat index as the parent dir:
///    - `/docs/index.html` ~ `/docs/index.txt` => `/docs`
pub fn normalize_http(path: &Path) -> PathBuf {
	// Handle empty path
	if path.as_os_str().is_empty() {
		return PathBuf::from("/");
	}

	// Get the file name (last component)
	let file_name = match path.file_name() {
		Some(name) => name,
		None => {
			// No file component (e.g., root "/"), just strip trailing slashes
			let s = path.to_string_lossy();
			return if s.len() <= 1 {
				PathBuf::from("/")
			} else {
				PathBuf::from(s.trim_end_matches('/'))
			};
		}
	};

	// Get the stem (filename without extension)
	let stem = path.file_stem().unwrap_or(file_name);
	let stem_str = stem.to_string_lossy();

	// Get parent directory
	let parent = path.parent().unwrap_or(Path::new("/"));

	// Rule 3: if stem is "index", return just the parent
	if stem_str == INDEX {
		let mut result = parent.to_path_buf();
		if result.as_os_str().is_empty() {
			result = PathBuf::from("/");
		}
		return result;
	}

	// Rules 1 & 2: return parent + stem (without extension)
	let mut result = parent.to_path_buf();
	result.push(stem_str.as_ref());
	result
}
