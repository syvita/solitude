use solitude::{Session, SessionStyle};

use anyhow::Result;

use env_logger::Target;

fn init() {
	let _ = env_logger::builder().is_test(true).format_module_path(true).target(Target::Stdout).try_init();
}

#[test]
fn service_can_be_resolved() -> Result<()> {
	init();

	let test_name = "service_can_be_resolved";

	let (session, mut second_session) = create_two_sessions(test_name, SessionStyle::Datagram, 0, 0)?;

	let session_address = session.address()?;
	let name = second_session.look_up(session_address.clone())?;
	println!("resolved {} to {}", session_address, name);

	Ok(())
}

fn create_two_sessions(test_name: &str, session_style: SessionStyle, first_port: u16, second_port: u16) -> Result<(Session, Session)> {
	let mut first_session = Session::new(format!("{}_first", test_name), session_style)?;
	first_session.forward(String::from("127.0.0.1"), first_port)?;

	let mut second_session = Session::new(format!("{}_second", test_name), session_style)?;
	second_session.forward(String::from("127.0.0.1"), second_port)?;

	Ok((first_session, second_session))
}
