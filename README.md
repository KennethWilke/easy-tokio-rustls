# Easy Tokio Rustls

This library provides convenient abstractions for creating simple TLS sockets with `tokio-rustls`.

Server sockets not yet implemented.

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
