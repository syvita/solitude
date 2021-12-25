use solitude::Session;

fn main() {
	let arguments: Vec<String> = std::env::args().collect();

	if arguments.len() < 2 {
		panic!("must supply I2P hostname, i.e. eva example.i2p");
	}

	let hostname = arguments[1].clone();

	println!("{}", hostname);

	let mut session = Session::new("echo_client".to_string()).unwrap();

	let destination = session.look_up(hostname).unwrap();

	session.send_to(destination, String::from("Hello World")).unwrap();
}
