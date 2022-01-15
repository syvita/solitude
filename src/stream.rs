use std::io::{BufRead, BufReader};

use crate::*;

// TODO: add FROM_PORT and TO_PORT
pub struct StreamInfo {
	pub destination: String,
}

impl StreamInfo {
	pub fn from_bufread<T: BufRead>(stream: &mut T) -> Result<Self> {
		debug!("deserializing stream info");

		let mut reader = BufReader::new(stream);

		let mut header = String::new();
		reader.read_line(&mut header)?;

		let expression = regex::Regex::new("^[^ ]+")?;

		let destination = expression
			.captures(&header)
			.context("Could not regex header")?
			.get(0)
			.context("Could not find destination in header")?
			.as_str()
			.to_owned();

		Ok(Self { destination })
	}
}
