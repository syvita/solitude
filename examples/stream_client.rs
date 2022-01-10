use std::io::Write;

use solitude::{Session, SessionStyle};

use anyhow::Result;

fn main() -> Result<()> {
	env_logger::builder()
		.filter_level(log::LevelFilter::Info)
		.parse_env("RUST_LOG")
		.init();

	let arguments: Vec<String> = std::env::args().collect();

	if arguments.len() < 2 {
		panic!("must supply I2P hostname, i.e. eva example.i2p");
	}

    let server_name = arguments[1].to_owned();

    let mut session = Session::new("stream_client_example".to_owned(), SessionStyle::Stream)?;
    let destination = session.look_up(server_name)?;

    let mut tcp_stream = session.connect_stream(destination)?;
    write!(tcp_stream, "Hello World!")?;

    Ok(())
}
