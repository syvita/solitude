use std::{io::{BufRead, BufReader}, net::TcpListener};

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
            let mut reader = BufReader::new(&mut stream);
            let stream_info = StreamInfo::from_bufread(&mut reader)?;

            let mut data = String::new();
            reader.read_line(&mut data)?;
            
            println!("received: \"{}\" from {}", data, stream_info.destination);
        }
    }

    Ok(())
}
