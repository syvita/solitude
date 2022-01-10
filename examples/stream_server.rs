use std::{io::{BufRead, BufReader}, net::TcpListener};

#[macro_use]
extern crate log;

use solitude::{Session, SessionStyle, StreamInfo};

use anyhow::Result;

fn main() -> Result<()> {
	env_logger::builder()
		.filter_level(log::LevelFilter::Info)
		.parse_env("RUST_LOG")
		.init();

    info!("Creating tcp server");
    let tcp_listener = TcpListener::bind("127.0.0.1:0")?;
    let port = tcp_listener.local_addr()?.port();

    info!("Creating SAMv3 session");
    let mut session = Session::new("stream_server_example".to_owned(), SessionStyle::Stream)?;
    info!("Forwarding tcp server to i2p");
    session.forward("127.0.0.1".to_owned(), port)?;

    info!("Listening on {}", session.address()?);

    for stream in tcp_listener.incoming() {
        if let Ok(mut stream) = stream {
            let mut reader = BufReader::new(&mut stream);
            let stream_info = StreamInfo::from_bufread(&mut reader)?;

            let mut data = String::new();
            reader.read_line(&mut data)?;
            
            info!("received: \"{}\" from {}", data, stream_info.destination);
        }
    }

    Ok(())
}
