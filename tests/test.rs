use solitude::{DatagramMessage, Session};

use std::net::UdpSocket;

use anyhow::{Context, Result};

fn init() {
	let _ = env_logger::builder().is_test(true).format_module_path(true).try_init();
}

#[test]
fn can_create_session() -> Result<()> {
	init();

	let test_name = "can_create_session ".to_string();

	let mut session = Session::new(test_name, "0.0.0.0", 0)?;

	session.close()?;
	Ok(())
}

#[test]
fn service_can_be_resolved() -> Result<()> {
	init();

	let test_name = "service_can_be_resolved";

	let (mut session, mut second_session) = create_two_sessions(test_name, 0, 0)?;

	let session_address = session.address()?;
	let name = second_session.look_up(session_address.clone())?;
	println!("resolved {} to {}", session_address, name);

	session.close()?;
	second_session.close()?;

	Ok(())
}

#[test]
fn can_send_datagram_to_service() -> Result<()> {
	init();

	let test_name = "can_send_data_to_service".to_string();

	let (udp_socket, second_udp_socket) = create_two_udp_sockets()?;

	let (mut session, mut second_session) =
		create_two_sessions(&test_name, udp_socket.local_addr()?.port(), second_udp_socket.local_addr()?.port())?;

	let destination = second_session.look_up("ME".to_string())?;

	let datagram_message = DatagramMessage::new(&test_name, &destination, [0x05, 0x15].to_vec());
	let datagram_message_bytes = datagram_message.serialize();

	udp_socket.send(&datagram_message_bytes)?;

	session.close()?;
	second_session.close()?;

	Ok(())
}

#[test]
fn can_create_datagram_message() -> Result<()> {
	init();

	let contents: [u8; 32] = rand::random();
	let datagram_message = DatagramMessage::new("test", "test_destination", contents.to_vec());

	Ok(())
}

#[test]
fn can_serialize_datagram_message() -> Result<()> {
	init();

	let contents: [u8; 32] = rand::random();
	let datagram_message = DatagramMessage::new("test", "test_destination", contents.to_vec());
	let _datagram_message_bytes = datagram_message.serialize();

	Ok(())
}

#[test]
fn can_deserialize_datagram_message() -> Result<()> {
	init();

	let example_received_datagram_bytes = [
		0x44, 0x41, 0x54, 0x41, 0x47, 0x52, 0x41, 0x4d, 0x20, 0x52, 0x45, 0x43, 0x45, 0x49, 0x56, 0x45, 0x44, 0x20, 0x44, 0x45, 0x53, 0x54,
		0x49, 0x4e, 0x41, 0x54, 0x49, 0x4f, 0x4e, 0x3d, 0x64, 0x65, 0x73, 0x74, 0x69, 0x6e, 0x61, 0x74, 0x69, 0x6f, 0x6e, 0x20, 0x53, 0x49,
		0x5a, 0x45, 0x3d, 0x6e, 0x75, 0x6d, 0x42, 0x79, 0x74, 0x65, 0x73, 0x0a, 0x05, 0x10,
	];

	let datagram_message_after_deserialization = DatagramMessage::from_bytes("test", &example_received_datagram_bytes)?;

	assert_eq!(datagram_message_after_deserialization.contents, [0x05, 0x10].to_vec());
	assert_eq!(datagram_message_after_deserialization.destination, "destination");

	Ok(())
}

fn create_two_udp_sockets() -> Result<(UdpSocket, UdpSocket)> {
	let first_udp_socket = UdpSocket::bind("0.0.0.0:0")?;
	let second_udp_socket = UdpSocket::bind("0.0.0.0:0")?;

	first_udp_socket.connect("127.0.0.1:7655")?;
	second_udp_socket.connect("127.0.0.1:7655")?;

	Ok((first_udp_socket, second_udp_socket))
}

fn create_two_sessions(test_name: &str, first_port: u16, second_port: u16) -> Result<(Session, Session)> {
	let session = Session::new(test_name.to_owned(), "0.0.0.0", first_port)?;

	let test_child_name = [test_name.to_owned(), "_child".to_string()].concat();
	let second_session = Session::new(test_child_name, "0.0.0.0", second_port)?;

	Ok((session, second_session))
}
