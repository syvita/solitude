use solitude::{DatagramMessage, Session, SessionStyle};

use std::net::{TcpListener, TcpStream};

use anyhow::Result;

fn init() {
	let _ = env_logger::builder().is_test(true).format_module_path(true).try_init();
}

#[test]
fn can_create_stream_session() -> Result<()> {
    init();

    let test_name = "can_create_stream_session".to_owned();

    let tcp_listener = TcpListener::bind("127.0.0.1:0")?;

    let _session = Session::new(test_name, SessionStyle::Stream, "127.0.0.1", tcp_listener.local_addr()?.port())?;

    Ok(())
}

