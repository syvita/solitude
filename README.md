# SOLITUDE (WIP)

i2p SAMv3 library written in Rust for use in the EVA project.

### Examples
#### Echo Server
Creates a UDP service that returns and prints whatever is sent to it. 
```sh
cargo run --example echo_server
```

#### Echo Client
Connects to a UDP service and sends data to it. 
```sh
cargo run --example echo_client *server_address
```
