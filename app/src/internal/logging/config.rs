use core::fmt;
use std::{fmt::Display, io, path::PathBuf, str::FromStr};

use clap::ValueEnum;

use crate::internal::util;

const DEFAULT_LOGFILE: &str = "fiche-n-chips.log";

#[derive(Clone, Debug, ValueEnum, strum::Display)]
pub enum LogFmt {
	Plain,
	Json,
	None,
}

#[derive(Clone)]
pub enum LogOutput {
	Stdout,
	Stderr,
	File(PathBuf),
}

impl FromStr for LogOutput {
	type Err = io::Error;
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		let output: Result<Self, io::Error> = match s.trim() {
			"stdout" | "-" => Ok(Self::Stdout),
			"stderr" | "err" => Ok(Self::Stderr),
			f => Ok(Self::File(util::valid_filepath(f, DEFAULT_LOGFILE)?)),
		};
		output
	}
}

impl Display for LogOutput {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			LogOutput::Stdout => f.write_str("stdout"),
			LogOutput::Stderr => f.write_str("stderr"),
			LogOutput::File(path) => f.write_str(path.to_str().unwrap_or_default()),
		}
	}
}
