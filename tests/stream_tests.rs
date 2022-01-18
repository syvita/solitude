#[macro_use]
extern crate log;

use solitude::{Session, SessionStyle};

use std::io::{BufRead, BufReader, Write};
use std::net::TcpListener;
use std::{time::Duration, thread};

use anyhow::Result;

use env_logger::Target;

fn init() {
	let _ = env_logger::builder().is_test(true).format_module_path(true).target(Target::Stdout).try_init();
	
	thread::sleep(Duration::from_secs(10));
}

#[test]
fn can_create_stream_forwarding_session() -> Result<()> {
	init();

	let tcp_listener = TcpListener::bind("127.0.0.1:0")?;

	let mut session = Session::new("can_create_stream_forwarding_session", SessionStyle::Stream)?;
	session.forward("127.0.0.1", tcp_listener.local_addr()?.port())?;

	Ok(())
}

#[test]
fn client_stream_can_send_to_listening_stream() -> Result<()> {
	init();

	let test_name = "client_stream_can_send_to_listening_stream";

	let tcp_listener = TcpListener::bind("127.0.0.1:0")?;
	let port = tcp_listener.local_addr()?.port();

	thread::spawn(move || {
		debug!("awaiting connections");
		for stream in tcp_listener.incoming() {
			debug!("received connection");

			match stream {
				Ok(stream) => {
					let mut buffer = String::new();
					let mut reader = BufReader::new(stream);
					reader.read_line(&mut buffer).unwrap();
					debug!("Received message: {}", buffer);
				}
				Err(e) => {
					debug!("failed connection: {:?}", e);
				}
			};
		}
	});

	let mut session = Session::new(test_name, SessionStyle::Stream)?;
	session.forward("127.0.0.1", port)?;

	let client_stream_session_name = format!("{}_client", test_name);

	let mut client_stream = Session::new(client_stream_session_name, SessionStyle::Stream)?;
	let mut tcp_stream = client_stream.connect_stream(session.public_key)?;

	write!(tcp_stream, "Hello World!")?;

	Ok(())
}

