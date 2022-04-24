# Easy Tokio Rustls

This library provides convenient abstractions for creating simple TLS sockets with `tokio-rustls`.

## Example client usage

```rust
use anyhow::Result;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use easy_tokio_rustls::TlsClient;

const BUFFER_SIZE: usize = 8 * 1024;
const REQUEST: &[u8] = b"GET / HTTP/1.1\r\nHost: suchprogramming.com\r\n\r\n";

#[tokio::main]
async fn main() -> Result<()> {
    let client = TlsClient::new("suchprogramming.com", 443).await?;
    let mut connection = client.connect().await?;

    connection.write_all(REQUEST).await?;
    let mut buffer = [0; BUFFER_SIZE];
    loop {
        let read_size = connection.read(&mut buffer).await?;
        if read_size == 0 {
            connection.shutdown().await?;
            return Ok(());
        }
        let html = std::str::from_utf8(&buffer[0..read_size]).unwrap();
        print!("{}", html);
        if html.contains("</html>") {
            connection.shutdown().await?;
            return Ok(());
        }
    }
}
```

## Example server usage

```rust
use anyhow::Result;
use std::str;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use easy_tokio_rustls::TlsServer;

const BUFFER_SIZE: usize = 8 * 1024;
const RESPONSE: &[u8] = b"HTTP/1.1 200 OK\r\nServer: a very great server\r\n\r\n";

#[tokio::main]
pub async fn main() -> Result<()> {
    let interface = "0.0.0.0";
    let port = 8443;
    let cert_file = "/home/ubuntu/fullchain.pem";
    let key_file = "/home/ubuntu/privkey-rsa.pem";

    let server = TlsServer::new(interface, port, cert_file, key_file).await?;
    let listener = server.listen().await?;
    println!("Listening on {}:{}", interface, port);

    // This is a simplified server, handling 1 connection at a time certainly isn't recommended
    let (stream, addr) = listener.stream_accept().await?;
    println!("Client connected from {}", addr);

    let mut client = stream.tls_accept().await?;
    println!("TLS connection accepted");

    let mut buffer = [0; BUFFER_SIZE];
    let read_size = client.read(&mut buffer).await?;
    let request = str::from_utf8(&buffer[..read_size])?;
    println!("Client sent:\n{}", request);

    client.write_all(RESPONSE).await?;
    client.flush().await?;
    println!("Reply sent, shutting down...");

    client.shutdown().await?;

    Ok(())
}
```

# Future features

Things I'd try to add to this project:

* mTLS Auth
* Certificate Pinning
