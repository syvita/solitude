use solitude::Session;

use anyhow::{Context, Result};

fn init() {
    let _ = env_logger::builder().is_test(true).format_module_path(true).try_init();
}



#[test]
fn can_create_session () -> Result<()> {
    init();

	let test_name = "can_create_session ".to_string();

	let mut session = Session::new(test_name)?;

    session.close()?;
	Ok(())
}

#[test]
fn service_can_be_resolved() -> Result<()> {
    init();

	let test_name = "service_can_be_resolved".to_string();

	let mut session = Session::new(test_name.clone())?;

	let test_child_name = [test_name, "_child".to_string()].concat();
	let mut new_session = Session::new(test_child_name)?;

	let session_address = session.address()?;
	let name = new_session.look_up(session_address.clone())?;
	println!("resolved {} to {}", session_address, name);

    session.close()?;
    new_session.close()?;

	Ok(())
}

#[test]
fn can_send_data_to_service() -> Result<()> {
    init();

	let test_name = "can_send_data_to_service".to_string();

    let (mut session, mut second_session) = create_two_sessions(test_name)?;

	let destination = session.look_up("ME".to_string())?;

	second_session.send_to(destination, "Hello".to_string())?;

    session.close()?;
    second_session.close()?;

	Ok(())
}

fn create_two_sessions(test_name: String) -> Result<(Session, Session)> {
	let session = Session::new(test_name.clone())?;

	let test_child_name = [test_name, "_child".to_string()].concat();
	let second_session = Session::new(test_child_name)?;

    Ok((session, second_session))
}
