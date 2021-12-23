use std::thread;

use i2p::Tunnel;

use anyhow::{Result, Context};

#[test]
fn can_create_tunnel() -> Result<()> {
    let test_name = "can_create_tunnel".to_string();

    let _tunnel = Tunnel::new(test_name).context("Couldn't create tunnel. Is i2pd or i2prouter running? Make sure SAMv3 is enabled.")?;

    Ok(())
}

#[test]
fn service_can_be_resolved() -> Result<()> {
    let test_name = "can_execute_hello".to_string();

    let tunnel = Tunnel::new(test_name)?;
        
    let mut new_tunnel = Tunnel::new("can_execute_hello_child".to_string())?;
    
    new_tunnel.look_up(tunnel.address())?;

    Ok(())
}

#[test]
fn can_connect_to_service() -> Result<()> {
    let test_name = "can_connect_to_service".to_string();

    let tunnel = Tunnel::new(test_name.clone())?;
        
    let test_child_name = [test_name, "_child".to_string()].concat();
    let mut new_tunnel = Tunnel::new(test_child_name)?;
    
    let destination = new_tunnel.look_up(tunnel.address())?;

    new_tunnel.send_to(destination, "Hello".to_string())?;

    Ok(())
}
