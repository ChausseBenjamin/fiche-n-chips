use std::collections::HashMap;
use std::io::Write;
use std::path::{Path, PathBuf};

use brotli::CompressorWriter;
use rslog::{attrs, warn};

use super::file::{embedded_files, WebFile};
use super::ReservedPath;
use crate::config::logging::{COUNT_KEY, HTML_ROOT_KEY, LIMIT_KEY, PATH_KEY, SIZE_KEY};
use crate::config::Config;
use crate::reserved::api::api_paths;
use crate::util::normalize_http;

const CONSIDERABLE_PATH_COUNT: u32 = 20;
const CONSIDERABLE_PATH_SIZE: usize = 1024 * 2;
const CONSIDERABLE_TOTAL_SIZE: usize = 1024 * 10;

pub struct ReservedPathProvider {
	files: HashMap<PathBuf, ReservedPath>,
}

impl ReservedPathProvider {
	pub fn new(cfg: &Config) -> Self {
		let mut files: HashMap<PathBuf, ReservedPath> =
			match cfg.server.html_dir.as_ref() {
				Some(p) => Self::load_files(p)
					.into_iter()
					.map(|(k, f)| (k, ReservedPath::File(f)))
					.collect(),
				None => embedded_files()
					.into_iter()
					.map(|f| {
						let k = f.path().to_path_buf();
						(k, ReservedPath::File(f))
					})
					.collect(),
			};

		for api in api_paths() {
			let k = api.path().to_path_buf();
			files.insert(k, api);
		}

		Self { files }
	}

	fn load_files(root: &Path) -> HashMap<PathBuf, WebFile<'static>> {
		let mut files = HashMap::new();
		let mut path_count: u32 = 0;
		let mut total_size: usize = 0;
		let mut has_index = false;
		let mut has_404 = false;

		let mut stack = vec![root.to_path_buf()];
		while let Some(dir) = stack.pop() {
			let Ok(entries) = std::fs::read_dir(&dir) else {
				continue;
			};
			for entry in entries.flatten() {
				let path = entry.path();
				if path.is_dir() {
					stack.push(path);
				} else if path.is_file() {
					path_count += 1;
					if path_count > CONSIDERABLE_PATH_COUNT {
						warn(
							"path count exceeds what seems reasonable. proceed if this is intended...",
							attrs!(
                COUNT_KEY => path_count,
                LIMIT_KEY => CONSIDERABLE_PATH_COUNT),
            );
					}

					let should_compress = matches!(
						path.extension().and_then(|e| e.to_str()),
						Some("html" | "svg" | "css" | "js")
					);

					let rel = path.strip_prefix(root).unwrap_or(&path);
					let web_path = normalize_http(&Path::new("/").join(rel));

					has_index |= web_path == Path::new("/");
					has_404 |= web_path == Path::new("/404");

					let Ok(content) = std::fs::read(&path) else {
						continue;
					};
					let file_size = content.len();
					total_size += file_size;

					let (compressed, bytes) = if should_compress {
						if file_size > CONSIDERABLE_PATH_SIZE {
							warn(
								"file size in the html dir exceeds what seems reasonable. proceed if this is intended...",
								attrs!(
                  PATH_KEY => path.display().to_string(),
                  SIZE_KEY => file_size),
              );
						}
						let mut out = Vec::new();
						{
							let mut w = CompressorWriter::new(&mut out, 4096, 11, 22);
							let _ = w.write_all(&content);
						}
						(true, out)
					} else {
						(false, content)
					};

					files.insert(
						web_path.clone(),
						WebFile::new(web_path, bytes, compressed),
					);
				}
			}
		}

		if total_size > CONSIDERABLE_TOTAL_SIZE {
			warn(
				"total size of the html dir exceeds what seems, reasonable. proceed if this is intended...",
				attrs!(
          SIZE_KEY => total_size,
          HTML_ROOT_KEY => format!("{}", root.display())),
      );
		}

		if !has_index {
			warn(
				"no index.html found in the html dir. the server will still run but will not have a homepage to serve on \"/\"",
				attrs!(HTML_ROOT_KEY => format!("{}", root.display())),
			);
		}

		if !has_404 {
			warn(
				"no 404.html found in the html dir. the server will still run but will not serve custom not-found pages",
				attrs!(HTML_ROOT_KEY => format!("{}", root.display())),
			);
		}

		files
	}

	pub fn paths(&self) -> Vec<&ReservedPath> { self.files.values().collect() }

	pub fn get(&self, path: &Path) -> Option<&ReservedPath> {
		self.files.get(path)
	}
}
