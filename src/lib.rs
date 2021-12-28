use std::{
	io::{BufRead, BufReader, Write},
	net::TcpStream,
	time::Duration,
};

#[macro_use]
extern crate anyhow;

#[macro_use]
extern crate log;

use anyhow::{Context, Result};
use data_encoding::{Specification, BASE32};
use sha2::{Digest, Sha256};

mod datagram;
pub use datagram::DatagramMessage;

mod stream;
pub use stream::StreamInfo;

/// Creates a SAMv3 session with local i2p daemon.
///
/// Forwards all connections to a server supplied by the user.
#[derive(Debug)]
pub struct Session {
	reader: BufReader<TcpStream>,
	stream: TcpStream,
	session_style: SessionStyle,
	pub public_key: String,
	private_key: String,
	pub service: String,
}

impl Session {
    /// Creates a new session which forwards to the supplied address.
    ///
    /// Should be used for Datagram, Raw, and StreamListener.
	pub fn new_forwarding_session(service: String, session_style: SessionStyle, forwarding_address: &str, forwarding_port: u16) -> Result<Self> {
		debug!("creating new forwarding session with ID {}", service);

        let mut session = Self::create_bare_session(service, session_style)?;

        session.keys()?;
		session.bridge(forwarding_address, forwarding_port)?;

		info!("Created new SAMv3 session");

		Ok(session)
	}

    /// Returns a stream that is connected to the supplied destination
    pub fn new_client_stream(service: String, destination: String) -> Result<TcpStream> {
        debug!("Creating new client stream");

        let mut session = Self::create_bare_session(service, SessionStyle::StreamClient)?;
        session.keys()?;

        session.command(&format!(
            "SESSION CREATE STYLE=STREAM ID={} DESTINATION={}",
            session.service, session.private_key,
        ))?;

        session.command(&format!(
            "STREAM CONNECT ID={} DESTINATION={}",
            session.service, destination,
        ))?;

        Ok(session.stream)
    }

    /// Creates a session that has only done HELLO.
    fn create_bare_session(service: String, session_style: SessionStyle) -> Result<Self> {
        trace!("creating unbridged session with id {}", service);

		let stream = TcpStream::connect("localhost:7656").context("couldn't connect to local SAM bridge")?;
		stream.set_read_timeout(Some(Duration::from_secs(500)))?;

		let mut session = Session {
			reader: BufReader::new(stream.try_clone()?),
			stream,
			session_style: session_style.to_owned(),
			public_key: String::new(),
			private_key: String::new(),
			service,
		};

		session.hello()?;

        Ok(session)
    }

	fn hello(&mut self) -> Result<()> {
		debug!("sam connection with ID {} is executing hello", self.service);

		let expression = regex::Regex::new(r#"HELLO REPLY RESULT=OK VERSION=(.*)\n"#)?;

		if !expression.is_match(&self.command("HELLO VERSION MIN=3.0 MAX=3.2\n")?) {
			bail!("didn't receive a hello response from i2p");
		}

		Ok(())
	}

	fn keys(&mut self) -> Result<()> {
		debug!("sam connection with ID {} is getting keys", self.service);

		let expression = regex::Regex::new(r#"DEST REPLY PUB=(?P<public>[^ ]*) PRIV=(?P<private>[^\n]*)"#)?;

		let body = &self.command("DEST GENERATE\n")?;
		let matches = expression.captures(body).context("invalid response")?;

		self.public_key = matches.name("public").context("no public key")?.as_str().to_string();
		self.private_key = matches.name("private").context("no private key")?.as_str().to_string();

		Ok(())
	}

	fn bridge(&mut self, forwarding_address: &str, port: u16) -> Result<()> {
		debug!("sam connection with ID {} is making a bridge", self.service);

		match self.session_style {
			SessionStyle::Datagram | SessionStyle::Raw => {
				self.command(&format!(
					"SESSION CREATE STYLE={} ID={} DESTINATION={} PORT={} HOST={}\n",
					self.session_style.to_string(),
					&self.service,
					&self.private_key,
					port,
					forwarding_address
				))?;
			}
			SessionStyle::StreamListener => {
				self.command(&format!(
					"SESSION CREATE STYLE=STREAM ID={} DESTINATION={}\n",
					self.service, self.private_key
				))
				.context("Could not create session")?;

                let mut session_to_send_forward_command = Session::create_bare_session("none".to_owned(), SessionStyle::StreamListener).context("Couldn't create session that executes STREAM FORWARD")?;

				session_to_send_forward_command.command(&format!(
					"STREAM FORWARD ID={} PORT={} HOST={}",
					self.service,
					port.to_string(),
					forwarding_address
				))
				.context("Could not forward session")?;
			}
            _ => {
                bail!("session_style must be StreamListener, Datagram, or Raw to bridge");
            }
		};

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

		let expression = regex::Regex::new(r#"(REPLY|STATUS)\s(RESULT=(?P<result>[^ ]*)(.*)|([^\n]*))"#)?;

		let matches = expression.captures(&response).context("Could not regex SAMv3's response")?;
		
		if let Some(result) = matches.name("result") {
			let plain = result.as_str();
			
			if plain == "OK" {
				Ok(response)
			} else {
				bail!(response)
			}
		} else {
			Ok(response)
		}
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

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SessionStyle {
	Datagram,
	Raw,
	StreamListener,
    StreamClient,
}

impl SessionStyle {
	fn to_string(&self) -> &str {
		match self {
			Self::Datagram => "DATAGRAM",
			Self::Raw => "RAW",
			Self::StreamListener | Self::StreamClient => "STREAM",
		}
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
