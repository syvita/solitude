#[macro_use]
extern crate log;

use solitude::{DatagramMessage, Session, SessionStyle};

use std::{net::TcpListener, io::{Write, BufReader, BufRead}, thread};

use anyhow::Result;

fn init() {
	let _ = env_logger::builder().is_test(true).format_module_path(true).try_init();
}

#[test]
fn can_create_stream_forwarding_session() -> Result<()> {
    init();

    let test_name = "can_create_stream_forwarding_session".to_owned();

    let tcp_listener = TcpListener::bind("127.0.0.1:0")?;

    let _session = Session::new_forwarding_session(test_name, SessionStyle::StreamListener, "127.0.0.1", tcp_listener.local_addr()?.port())?;

    Ok(())
}

#[test]
fn client_stream_can_send_to_listening_stream() -> Result<()> {
    init();

    let test_name = "client_stream_can_send_to_listening_stream".to_owned();

    let tcp_listener = TcpListener::bind("127.0.0.1:0")?;
    let port = tcp_listener.local_addr()?.port();

    thread::spawn(move || {
        for stream in tcp_listener.incoming() {
            match stream {
                Ok(stream) => {
                    let mut buffer = String::new();
                    let mut reader = BufReader::new(stream);
                    reader.read_line(&mut buffer).unwrap();
                    debug!("Received message: {}", buffer);
                    
                    // TODO
                }
                Err(e) => {
                    debug!("failed connection: {:?}", e);
                }
            };
        };
    });

    let session = Session::new_forwarding_session(test_name.to_owned(), SessionStyle::StreamListener, "127.0.0.1", port)?;

    
    let mut tcp_stream = Session::new_client_stream(format!("{}_client", test_name), session.public_key)?;
    write!(tcp_stream, "Hello World!")?;

    Ok(())
}
