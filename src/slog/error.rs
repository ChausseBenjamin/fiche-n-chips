use std::{error::Error as StdError, fmt, io};

#[derive(Debug)]
pub enum HandlerError {
	Write(io::Error),
	Parse(String),
}
