use core::fmt;
use std::{
	fmt::Display,
	io::{self, Write},
	path::PathBuf,
	str::FromStr,
};

use clap::{Args, ValueEnum};
use rslog::{
	DiscardHandler, JsonHandler, Level, Logger, SlogLevel, TextHandler, DEBUG,
	ERROR, INFO, WARN,
};

use crate::internal::util;

const DEFAULT_LOGFILE: &str = "/var/log/fiche-n-chips.log";

#[derive(Args)]
#[clap(next_help_heading = "Logging")]
pub struct LoggingCfg {
	#[arg(long = "log-level", env = "LOG_LEVEL", default_value = "info")]
	/// Log verbosity level
	pub level: LogLevel,
	#[arg(long = "log-output", env = "LOG_OUTPUT", default_value = "stdout")]
	/// Log destination
	pub output: LogOutput,
	#[arg(long = "log-format", env = "LOG_FORMAT", default_value = "json")]
	/// Log output format
	pub format: LogFmt,
	#[arg(long = "log-trace", env = "LOG_TRACE", default_value_t = false)]
	/// Enable trace logging
	pub trace: bool,
}

impl<W: Write> TryFrom<LoggingCfg> for Logger {
	type Error = io::Error;
	fn try_from(value: LoggingCfg) -> Result<Self, Self::Error> {
		let lvl: i8 = value.level.into();
		let w: Write = match value.output {
			LogOutput::Stdout => std::io::Stdout,
			LogOutput::Stderr => std::io::Stderr,
			LogOutput::File(f) => todo!(),
		};
		Ok(match value.format {
			LogFmt::Plain => {
				TextHandler::new(w).min_level(lvl).attach_source(value.trace).into()
			}
			LogFmt::Json => {
				JsonHandler::new(w).min_level(lvl).attach_source(value.trace).into()
			}
			LogFmt::None => DiscardHandler {}.into(),
		})
	}
}

#[derive(Clone, Debug, ValueEnum)]
pub enum LogFmt {
	Plain,
	Json,
	None,
}

#[derive(Debug, Clone, ValueEnum)]
pub enum LogLevel {
	#[value(alias = "d")]
	Debug,
	#[value(alias = "i")]
	Info,
	#[value(alias = "w")]
	Warn,
	#[value(alias = "e")]
	Error,
}

impl From<LogLevel> for i8 {
	fn from(value: LogLevel) -> Self {
		match value {
			LogLevel::Debug => DEBUG.value(),
			LogLevel::Info => INFO.value(),
			LogLevel::Warn => WARN.value(),
			LogLevel::Error => ERROR.value(),
		}
	}
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
