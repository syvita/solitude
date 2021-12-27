use std::io::{BufRead, BufReader, Write};
use std::net::TcpStream;

#[macro_use]
extern crate anyhow;

#[macro_use]
extern crate log;

use anyhow::{Context, Result};
use data_encoding::{Specification, BASE32};
use sha2::{Digest, Sha256};

/// Creates a SAMv3 session with local i2p daemon.
///
/// Forwards all connections to a server supplied by the user.
#[derive(Debug)]
pub struct Session {
	reader: BufReader<TcpStream>,
	stream: TcpStream,
	pub public_key: String,
	private_key: String,
	pub service: String,
}

impl Session {
	pub fn new(service: String, forwarding_address: &str, forwarding_port: u16) -> Result<Self> {
		debug!("creating new session with ID {}", service);

		let stream = TcpStream::connect("localhost:7656").context("couldn't connect to local SAM bridge")?;

		let mut session = Session {
			reader: BufReader::new(stream.try_clone()?),
			stream,
			public_key: String::new(),
			private_key: String::new(),
			service,
		};

		session.hello()?;

		session.keys()?;

		session.bridge(forwarding_address, forwarding_port)?;

		info!("Created new SAMv3 session {:?}", session);

		Ok(session)
	}

	fn hello(&mut self) -> Result<()> {
		debug!("sam connection with ID {} executed hello", self.service);

		let expression = regex::Regex::new(r#"HELLO REPLY RESULT=OK VERSION=(.*)\n"#)?;

		if !expression.is_match(&self.command("HELLO VERSION MIN=3.0 MAX=3.2\n")?) {
			bail!("didn't receive a hello response from i2p");
		}

		Ok(())
	}

	fn keys(&mut self) -> Result<()> {
		debug!("sam connection with ID {} got keys", self.service);

		let expression = regex::Regex::new(r#"DEST REPLY PUB=(?P<public>[^ ]*) PRIV=(?P<private>[^\n]*)"#)?;

		let body = &self.command("DEST GENERATE\n")?;
		let matches = expression.captures(body).context("invalid response")?;

		self.public_key = matches.name("public").context("no public key")?.as_str().to_string();
		self.private_key = matches.name("private").context("no private key")?.as_str().to_string();

		Ok(())
	}

	fn bridge(&mut self, forwarding_address: &str, port: u16) -> Result<()> {
		debug!("sam connection with ID {} made a new session", self.service);

		let expression = regex::Regex::new(r#"SESSION STATUS RESULT=OK DESTINATION=([^\n]*)"#)?;

		let body = &self.command(&format!(
			"SESSION CREATE STYLE=DATAGRAM ID={} DESTINATION={} PORT={} HOST={}\n",
			&self.service, &self.private_key, port, forwarding_address
		))?;

		if !expression.is_match(body) {
			bail!("didn't receive a hello response from i2p")
		}

		Ok(())
	}

	pub fn address(&self) -> Result<String> {
		let public_key_bytes = decode_base_64(&self.public_key)?;

		let mut hasher = Sha256::new();

		hasher.update(public_key_bytes);

		let address = BASE32.encode(&hasher.finalize()).to_lowercase();

		Ok(address.trim_end_matches('=').to_owned() + ".b32.i2p")
	}

	pub fn close(&mut self) -> Result<()> {
		debug!("sam connection with ID {} is closing session", self.service);

		self.command_with_no_response("QUIT")?;

		Ok(())
	}

	fn command_with_no_response(&mut self, command: &str) -> Result<()> {
		debug!("sam connection with ID {} is executing command {}", self.service, command);

		self.stream.write_all(command.as_bytes())?;

		Ok(())
	}

	fn command(&mut self, command: &str) -> Result<String> {
		debug!("sam connection with ID {} is executing command {}", self.service, command);

		self.stream.write_all(command.as_bytes())?;

		let mut response = String::new();

		self.reader.read_line(&mut response)?;
		trace!(
			"sam connection with ID {} sent command {} and got response {}",
			self.service,
			command,
			response
		);

		Ok(response)
	}

	pub fn look_up(&mut self, address: String) -> Result<String> {
		debug!("sam connection with ID {} is looking up address {}", self.service, address);

		let expression = regex::Regex::new(r#"NAMING REPLY RESULT=OK NAME=([^ ]*) VALUE=(?P<value>[^\n]*)\n"#)?;

		let body = self.command(&format!("NAMING LOOKUP NAME={}\n", address))?;

		let matches = expression.captures(&body).context("could not resolve domain")?;

		let value = matches
			.name("value")
			.context("no return value, possibly an invalid domain")?
			.as_str()
			.to_string();

		Ok(value)
	}
}

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

		let header_bytes = split_buffer.iter().nth(0).context("Cannot deserialize an empty buffer")?;

		let header = String::from_utf8(header_bytes.to_vec())?;

		let expression = regex::Regex::new(r#"DATAGRAM RECEIVED DESTINATION=(?P<destination>[^ ]*) SIZE=(?P<size>[^\n]*)"#)?;

		let matches = expression.captures(&header).context("Could not find fields in received datagram")?;

		let destination = matches
			.name("destination")
			.context("Could not find destination in received datagram")?
			.as_str()
			.to_owned();

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

fn decode_base_64(base_64_code: &str) -> Result<Vec<u8>> {
	let mut specification = Specification::new();
	specification
		.symbols
		.push_str("ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-~");

	let encoder = specification.encoding().unwrap();
	let buffer = encoder.decode(base_64_code.as_bytes())?;

	Ok(buffer)
}
