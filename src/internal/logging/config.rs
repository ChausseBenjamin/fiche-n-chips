use std::{io, path::PathBuf, str::FromStr};

use crate::internal::util::valid_filepath;

const DEFAULT_LOGFILE: &str = "fiche-n-chips.log";

pub enum LogFmt {
	Plain,
	Json,
	None,
}

#[derive(Clone, Debug)]
pub enum LogOutput {
	Stdout,
	Stderr,
	File(PathBuf),
}

impl FromStr for LogOutput {
	type Err = io::Error;
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		let output: Result<Self, YourErrorType> = match s.trim() {
			"stdout" | "-" => Ok(Self::Stdout),
			"stderr" | "err" => Ok(Self::Stderr),
			v => Ok(Self::File(valid_filepath(v, DEFAULT_LOGFILE)?)),
		};
		output
	}
}
