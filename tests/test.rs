use solitude::Tunnel;

use anyhow::{Context, Result};

fn init() {
    let _ = env_logger::builder().is_test(true).try_init();
}

#[test]
fn can_create_tunnel() -> Result<()> {
    init();

	let test_name = "can_create_tunnel".to_string();

	let mut tunnel = Tunnel::new(test_name)?;

    tunnel.close()?;
	Ok(())
}

#[test]
fn service_can_be_resolved() -> Result<()> {
    init();

	let test_name = "service_can_be_resolved".to_string();

	let mut tunnel = Tunnel::new(test_name.clone())?;

	let test_child_name = [test_name, "_child".to_string()].concat();
	let mut new_tunnel = Tunnel::new(test_child_name)?;

	let tunnel_address = tunnel.address()?;
	let name = new_tunnel.look_up(tunnel_address.clone())?;
	println!("resolved {} to {}", tunnel_address, name);

    tunnel.close()?;
    new_tunnel.close()?;

	Ok(())
}

#[test]
fn can_send_data_to_service() -> Result<()> {
    init();

	let test_name = "can_send_data_to_service".to_string();

	let mut tunnel = Tunnel::new(test_name.clone())?;
	let destination = tunnel.look_up("ME".to_string())?;

	let test_child_name = [test_name, "_child".to_string()].concat();
	let mut new_tunnel = Tunnel::new(test_child_name)?;

	new_tunnel.send_to(destination, "Hello".to_string())?;

    tunnel.close()?;
    new_tunnel.close()?;

	Ok(())
}
