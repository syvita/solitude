use crate::*;

struct StreamInfo {
    pub destination: String,
}

impl StreamInfo {
    fn from_bytes(buffer: &[u8]) -> Result<Self> {
		debug!("deserializing datagram message");

		// Split the buffer, using the first 0x0a (newline) byte as the delimiter
		let split_buffer: Vec<&[u8]> = buffer.splitn(1, |byte| *byte == 0x0a).collect();

		let header_bytes = split_buffer.iter().nth(0).context("Cannot deserialize an empty buffer")?;

		let header = String::from_utf8(header_bytes.to_vec())?;

        let expression = regex::Regex::new("^[^ ]+")?;

        let destination = expression
            .captures(&header)
            .context("Could not regex header")?
            .get(0)
            .context("Could not find destination in header")?
            .as_str()
            .to_owned();

		Ok(Self {
			destination,
		})
    }
}
