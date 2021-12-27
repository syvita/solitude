#[macro_use]
extern crate log;

use solitude::{DatagramMessage, Session};

use std::net::UdpSocket;

fn main() {
	env_logger::init();

	let arguments: Vec<String> = std::env::args().collect();

	if arguments.len() < 2 {
		panic!("must supply I2P hostname, i.e. eva example.i2p");
	}

	let udp_socket = UdpSocket::bind("0.0.0.0:0").unwrap();
	udp_socket.connect("127.0.0.1:7655").unwrap();
	let port = udp_socket.local_addr().unwrap().port();

	let mut session = Session::new("echo_client".to_string(), "0.0.0.0", port).unwrap();

	let hostname = arguments[1].to_owned();

	let destination = session.look_up(hostname).unwrap();

	let datagram = DatagramMessage::new("echo_client", &destination, b"Hello World!".to_vec());
    info!("Sending datagram");
	debug!("datagram: {:x?}", datagram);

	let datagram_bytes = datagram.serialize();

    // Sends 10 datagrams over one second. Datagrams fail occasionally, this makes it likely that
    // at least on will go through
    for _ in 0..10 {
        std::thread::sleep(std::time::Duration::from_millis(100));

	    udp_socket.send(&datagram_bytes).unwrap();
	    info!("Sent datagram");
    }
}
