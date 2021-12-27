use std::net::UdpSocket;

use solitude::Session;

fn main() {
	let socket = UdpSocket::bind("0.0.0.0:0").expect("failed to bind");
	
	let port = socket.local_addr().expect("failed to get address").port();
	
	let session = Session::new(String::from("echo_server"), "0.0.0.0", port).expect("couldn't create session");

	println!("listening at 0.0.0.0:{} or {}", port, session.address().unwrap());

	let mut buffer = [0; 2048];

	loop {
		let (frame, source) = socket.recv_from(&mut buffer).unwrap();
		let buffer = &mut buffer[..frame];

		println!("from: {:?}, buffer: {:?}", source, buffer);

		socket.send_to(buffer, &source).unwrap();
	}
}
