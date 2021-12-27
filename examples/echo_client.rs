use std::net::UdpSocket;

use solitude::{Session, DatagramMessage};

fn main() {
	let arguments: Vec<String> = std::env::args().collect();

	if arguments.len() < 2 {
		panic!("must supply I2P hostname, i.e. eva example.i2p");
	}

	let hostname = arguments[1].clone();

	let socket = UdpSocket::bind("0.0.0.0:0").expect("failed to bind");
		
	socket.connect("127.0.0.1:7655").expect("failed to connect to I2P's UDP bridge");
	
	let mut session = Session::new("echo_client".to_string(), "0.0.0.0", 0).expect("failed to create Session");

	let destination = session.look_up(hostname).expect("failed to lookup hostname");
	
	let message = DatagramMessage::new("echo_client", &destination, String::from("Hello World").into_bytes());
	let raw = message.serialize();
	
	socket.send(&raw).expect("failed to send message");
}
