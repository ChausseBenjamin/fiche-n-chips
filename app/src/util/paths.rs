use std::{
	fs::{metadata, OpenOptions},
	io::{Error, ErrorKind},
	path::{Path, PathBuf},
};

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

	match metadata(&path) {
		Ok(meta) if meta.is_file() => {
			OpenOptions::new().append(true).open(&path)?;
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
