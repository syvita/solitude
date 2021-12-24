use solitude;

fn main() {
	let tunnel = solitude::Tunnel::new(String::from("echo_server")).expect("couldn't create tunnel");

	println!("listening at {}", tunnel.address().unwrap());

	let mut buffer = [0; 2048];

	loop {
		let (frame, source) = tunnel.socket.recv_from(&mut buffer).unwrap();
		let buffer = &mut buffer[..frame];

		println!("from: {:?}, buffer: {:?}", source, buffer);

		tunnel.socket.send_to(buffer, &source).unwrap();
	}
}