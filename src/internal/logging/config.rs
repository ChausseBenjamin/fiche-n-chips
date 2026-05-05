use core::fmt;
use std::{io, path::PathBuf, str::FromStr};

use clap::ValueEnum;

use crate::internal::util;

const DEFAULT_LOGFILE: &str = "fiche-n-chips.log";

#[derive(Clone, Debug, ValueEnum)]
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

impl ToString for LogOutput {
	fn to_string(&self) -> String {
		match self {
			LogOutput::Stdout => "stdout".into(),
			LogOutput::Stderr => "stderr".into(),
			LogOutput::File(path) => {
				if let Some(v) = path.to_str() {
					v.into()
				} else {
					"".to_string()
				}
			}
		}
	}
}
