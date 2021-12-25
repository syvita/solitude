use std::thread;

use solitude::Tunnel;

use anyhow::{Context, Result};

#[test]
fn can_create_tunnel() -> Result<()> {
	let test_name = "can_create_tunnel".to_string();

	let _tunnel = Tunnel::new(test_name).context("Couldn't create tunnel. Is i2pd or i2prouter running? Make sure SAMv3 is enabled.")?;

	Ok(())
}

#[test]
fn service_can_be_resolved() -> Result<()> {
	let test_name = "can_execute_hello".to_string();

	let tunnel = Tunnel::new(test_name.clone())?;

	let test_child_name = [test_name, "_child".to_string()].concat();
	let mut new_tunnel = Tunnel::new(test_child_name)?;

	let tunnel_address = tunnel.address()?;
	let name = new_tunnel.look_up(tunnel_address.clone())?;
	println!("resolved {} to {}", tunnel_address, name);

	Ok(())
}

#[test]
fn can_send_data_to_service() -> Result<()> {
	let test_name = "can_connect_to_service".to_string();

	let mut tunnel = Tunnel::new(test_name.clone())?;
	let destination = tunnel.look_up("ME".to_string())?;

	let test_child_name = [test_name, "_child".to_string()].concat();
	let mut new_tunnel = Tunnel::new(test_child_name)?;

	new_tunnel.send_to(destination, "Hello".to_string())?;

	Ok(())
}
