use std::{io::Read, net::TcpListener};

use solitude::{Session, SessionStyle, StreamInfo};

use anyhow::Result;

fn main() -> Result<()> {
	env_logger::builder()
		.filter_level(log::LevelFilter::Info)
		.parse_env("RUST_LOG")
		.init();

    let tcp_listener = TcpListener::bind("127.0.0.1:0")?;
    let port = tcp_listener.local_addr()?.port();

    let mut session = Session::new("stream_server_example".to_owned(), SessionStyle::Stream)?;
    session.forward("127.0.0.1".to_owned(), port)?;

    println!("listening on {}", session.address()?);

    for stream in tcp_listener.incoming() {
        if let Ok(mut stream) = stream {
            let mut buffer = [0u8; 2048];
            let length = stream.read(&mut buffer)?;
            let buffer = &buffer[..length];

            let stream_info = StreamInfo::from_bytes(&buffer)?;
            println!("received message from {}", stream_info.destination);
        }
    }

    Ok(())
}
