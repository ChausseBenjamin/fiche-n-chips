use std::io;

#[derive(Debug)]
pub enum HandlerError {
	Write(io::Error),
	Parse(String),
}
