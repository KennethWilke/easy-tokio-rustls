use anyhow::Result;
use std::str;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use easy_tokio_rustls::TlsServer;

const BUFFER_SIZE: usize = 8 * 1024;
const RESPONSE: &[u8] = b"HTTP/1.1 200 OK\r\nServer: a very great server\r\n\r\n";

#[tokio::main]
pub async fn main() -> Result<()> {
    let interface = "0.0.0.0:8443";
    let cert_file = "cert.pem";
    let key_file = "privkey.pem";

    let server = TlsServer::new(interface, cert_file, key_file).await?;
    let listener = server.listen().await?;
    println!("Listening on {}", interface);

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
