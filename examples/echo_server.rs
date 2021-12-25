use solitude::Session;

fn main() {
	let session = Session::new(String::from("echo_server")).expect("couldn't create session");

	println!("listening at {}", session.address().unwrap());

	let mut buffer = [0; 2048];

	loop {
		let (frame, source) = session.socket.recv_from(&mut buffer).unwrap();
		let buffer = &mut buffer[..frame];

		println!("from: {:?}, buffer: {:?}", source, buffer);

		session.socket.send_to(buffer, &source).unwrap();
	}
}
