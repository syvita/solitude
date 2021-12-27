use std::io::{BufRead, BufReader, Write};
use std::net::TcpStream;
use std::net::UdpSocket;

#[macro_use]
extern crate anyhow;

#[macro_use]
extern crate log;

use anyhow::{Context, Result};
use data_encoding::{Specification, BASE32};
use sha2::{Digest, Sha256};

#[derive(Debug)]
pub struct Session {
	reader: BufReader<TcpStream>,
	stream: TcpStream,

	pub socket: UdpSocket,

	pub public_key: String,
	private_key: String,

	pub service: String,
}

impl Session {
	pub fn new(service: String) -> Result<Self> {
        debug!("creating new session with ID {}", service);

		let stream = TcpStream::connect("localhost:7656").context("couldn't connect to local SAM bridge")?;

		let socket = UdpSocket::bind(("0.0.0.0", 0))?;
		let port = socket.local_addr()?.port();

		let mut session = Session {
			reader: BufReader::new(stream.try_clone()?),
			stream,
			socket,
			public_key: String::new(),
			private_key: String::new(),
			service,
		};

		session.hello()?;

		session.keys()?;

		session.new_session(port)?;

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

	fn new_session(&mut self, port: u16) -> Result<()> {
        debug!("sam connection with ID {} made a new session", self.service);

		let expression = regex::Regex::new(r#"SESSION STATUS RESULT=OK DESTINATION=([^\n]*)"#)?;

		let body = &self.command(&format!(
			"SESSION CREATE STYLE=DATAGRAM ID={} DESTINATION={} PORT={} HOST=0.0.0.0\n",
			&self.service, &self.private_key, port
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
		debug!("sam connection with ID {} is closing i2p", self.service);

		let body = &self.command("QUIT")?;

		if body != "QUIT STATUS RESULT=OK MESSAGE=bye" {
			bail!("failed to quit, are you using an up-to-date version of i2prouter?");
		}

		Ok(())
	}

	pub fn send_to(&mut self, address: String, message: String) -> Result<usize> {
        debug!("sam connection with ID {} is sending data to {}", self.service, address);

		let length = &self.socket.send_to(
			format!("3.1 {} {}\n{}", &self.service, address, message).as_bytes(),
			"127.0.0.1:7655",
		)?;

		Ok(*length)
	}

	fn command(&mut self, command: &str) -> Result<String> {
        debug!("sam connection with ID {} is executing command {}", self.service, command);

		self.stream.write_all(command.as_bytes())?;

		let mut response = String::new();

		self.reader.read_line(&mut response)?;
        trace!("sam connection with ID {} sent command {} and got response {}", self.service, command, response);

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

fn decode_base_64(base_64_code: &str) -> Result<Vec<u8>> {
	let mut specification = Specification::new();
	specification
		.symbols
		.push_str("ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-~");

	let encoder = specification.encoding().unwrap();
	let buffer = encoder.decode(base_64_code.as_bytes())?;

	Ok(buffer)
}
