# SOLITUDE (WIP)

i2p SAMv3 library written in Rust for use in the EVA project.

### Examples
#### Datagram Server
Creates a datagram service that logs whatever is sent to it.
```sh
cargo run --example datagram_server
```

#### Datagram client
Sends a datagram with the payload "Hello World! "to the supplied server
```sh
cargo run --example datagram_client <server_name>
```

#### Stream Server
Creates a Stream style server that logs whatever is sent to it.
```sh
cargo run --example stream_server
```

#### Stream Client
Sends "Hello World!" to the supplied server
```sh
cargo run --example stream_client <server_name>
```
