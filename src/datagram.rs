use crate::*;

#[derive(Debug, PartialEq)]
pub struct DatagramMessage {
	pub session_id: String,
	pub destination: String,
	pub contents: Vec<u8>,
}

impl DatagramMessage {
	pub fn new(session_id: &str, destination: &str, contents: Vec<u8>) -> Self {
		Self {
			session_id: session_id.to_owned(),
			destination: destination.to_owned(),
			contents,
		}
	}

	pub fn serialize(&self) -> Vec<u8> {
		debug!("serializing datagram message");

		let header = format!("3.0 {} {}\n", self.session_id, self.destination);
		let mut bytes = header.as_bytes().to_vec();
		bytes.append(&mut self.contents.clone());

		bytes
	}

	pub fn from_bytes(session_id: &str, buffer: &[u8]) -> Result<Self> {
		debug!("deserializing datagram message");

		// Split the buffer, using the first 0x0a (newline) byte as the delimiter
		let split_buffer: Vec<&[u8]> = buffer.splitn(2, |byte| *byte == 0x0a).collect();

		let destination_bytes = split_buffer.iter().nth(0).context("Cannot deserialize an empty buffer")?;

		let destination = String::from_utf8(destination_bytes.to_vec())?;

		let contents = split_buffer
			.iter()
			.nth(1)
			.context("could not find contents of datagram message")?
			.to_vec();

		Ok(Self {
			session_id: session_id.to_owned(),
			destination,
			contents,
		})
	}
}
