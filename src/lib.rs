use std::io::{BufRead, BufReader, Write, Read};
use std::net::{TcpStream, Shutdown};
use std::sync::mpsc::channel;
use std::thread;
use std::time::Duration;

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
	pub private_key: String,
	pub service: String,
}

impl Session {
	/// Creates a session that has only done HELLO.
	pub fn new<S: Into<String>>(service: S, session_style: SessionStyle) -> Result<Self> {
		let service_string = service.into();
		
		trace!("creating new session with id {}", service_string);

		let stream = TcpStream::connect("localhost:7656").context("couldn't connect to local SAM bridge")?;
		stream.set_read_timeout(Some(Duration::from_secs(90)))?;

		let mut session = Session {
			reader: BufReader::new(stream.try_clone()?),
			stream,
			session_style: session_style.to_owned(),
			public_key: String::new(),
			private_key: String::new(),
			service: service_string,
		};

		session.hello()?;
		session.keys()?;

		Ok(session)
	}
	
	pub fn from<S: Into<String>>(service: S, session_style: SessionStyle, public_key: S, private_key: S) -> Result<Self> {
		let service_string = service.into();
		let public_key_string = public_key.into();
		
		trace!("restoring session with id {} and public_key {}", service_string, public_key_string);

		let stream = TcpStream::connect("localhost:7656").context("couldn't connect to local SAM bridge")?;
		stream.set_read_timeout(Some(Duration::from_secs(90)))?;

		let mut session = Session {
			reader: BufReader::new(stream.try_clone()?),
			stream,
			session_style: session_style.to_owned(),
			public_key: public_key_string,
			private_key: private_key.into(),
			service: service_string,
		};

		session.hello()?;

		Ok(session)
	}

	pub fn forward<S: Into<String>>(&mut self, forwarding_address: S, port: u16) -> Result<()> {
		let forwarding_address_string = forwarding_address.into();
		
		debug!("sam connection with ID {} is forwarding", self.service);

		match self.session_style {
			SessionStyle::Datagram | SessionStyle::Raw => {
				self.command(&format!(
					"SESSION CREATE STYLE={} ID={} DESTINATION={} PORT={} HOST={}\n",
					self.session_style.as_string(),
					&self.service,
					&self.private_key,
					port,
					forwarding_address_string
				))?;
			}
			SessionStyle::Stream => {
				self.command(&format!(
					"SESSION CREATE STYLE={} ID={} DESTINATION={}\n",
					self.session_style.as_string(),
					self.service,
					self.private_key
				))
				.context("Could not create session")?;

				let (sender, receiver) = channel::<Result<_>>();

				let new_service = self.service.clone();

				thread::spawn(move || {
					let mut new_session = match Session::new(new_service.clone(), SessionStyle::Stream) {
						Ok(session) => session,
						Err(error) => {
							sender.send(Err(error)).unwrap();
							return;
						}
					};

					if let Err(error) = new_session.command(&format!(
						"STREAM FORWARD ID={} PORT={} HOST={}\n",
						new_service,
                        port,
						forwarding_address_string,
					)) {
						sender.send(Err(error)).unwrap();
						return;
					}

					sender.send(Ok(())).unwrap();

					loop {
				    	let mut buffer = [];
				    	let read = new_session.stream.read(&mut buffer);
						
				    	if let Err(error) = read {
				    		panic!("stream forwarder closed with: {}", error);
				    	}

						std::thread::sleep(Duration::from_secs(2));
					}
				});

				for _ in 0..1 {
					receiver.recv()??;
				}
			}
		};

		Ok(())
	}

	/// Returns a TcpStream connected to the destination.
	pub fn connect_stream<S: Into<String>>(&mut self, destination: S) -> Result<TcpStream> {
		self.command(&format!(
			"SESSION CREATE STYLE=STREAM ID={} DESTINATION={}\n",
			self.service, self.private_key,
		))
		.context("Couldn't create session")?;

		let mut connected_session = Session::new(self.service.to_owned(), SessionStyle::Stream)?;

		connected_session.command(&format!("STREAM CONNECT ID={} DESTINATION={}\n", self.service, destination.into()))?;

		Ok(connected_session.stream)
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

	pub fn address(&self) -> Result<String> {
		let public_key_bytes = decode_base_64(&self.public_key)?;

		let mut hasher = Sha256::new();

		hasher.update(public_key_bytes);

		let address = BASE32.encode(&hasher.finalize()).to_lowercase();

		Ok(address.trim_end_matches('=').to_owned() + ".b32.i2p")
	}

	pub fn close(self) -> Result<()> {
		debug!("sam connection with ID {} is closing i2p", self.service);

		self.stream.shutdown(Shutdown::Both)?;

		Ok(())
	}

	fn command(&mut self, command: &str) -> Result<String> {
		debug!("sam connection with ID {} is executing command {}", self.service, command);

		self.stream.write_all(command.as_bytes())?;

		let mut response = String::new();

		trace!("reading from SAM socket");
		self.reader.read_line(&mut response)?;
		trace!("read from SAM socket");

		trace!(
			"sam connection with ID {} sent command {} and got response {}",
			self.service,
			command,
			response
		);

		let expression = regex::Regex::new(r#"(REPLY|STATUS)\s(RESULT=(?P<result>[^\s]*)(.*)|([^\n]*))"#)?;

		let matches = expression.captures(&response).context("Could not regex SAMv3's response")?;

		if let Some(result) = matches.name("result") {
			let result_str = result.as_str();

			if result_str == "OK" {
				Ok(response)
			} else {
				bail!(response)
			}
		} else {
			Ok(response)
		}
	}

	pub fn look_up<S: Into<String>>(&mut self, address: S) -> Result<String> {
		let address_string = address.into();
		
		debug!("sam connection with ID {} is looking up address {}", self.service, address_string);

		let expression = regex::Regex::new(r#"NAMING REPLY RESULT=OK NAME=([^ ]*) VALUE=(?P<value>[^\n]*)\n"#)?;

		let body = self.command(&format!("NAMING LOOKUP NAME={}\n", address_string))?;

		let matches = expression.captures(&body).context("could not resolve domain")?;

		let value = matches
			.name("value")
			.context("no return value, possibly an invalid domain")?
			.as_str()
			.to_string();

		Ok(value)
	}
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum SessionStyle {
	Datagram,
	Raw,
	Stream,
}

impl SessionStyle {
	pub fn as_string(&self) -> &str {
		match self {
			Self::Datagram => "DATAGRAM",
			Self::Raw => "RAW",
			Self::Stream => "STREAM",
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
