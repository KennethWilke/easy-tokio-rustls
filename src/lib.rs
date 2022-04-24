//! This library provides convenient abstractions for creating simple TLS
//! sockets with `tokio-rustls`.

use anyhow::Result;
use std::net::{SocketAddr, ToSocketAddrs};

mod certstore;
mod client;
mod server;

pub use client::{TlsClient, TlsClientError};
pub use server::{TlsServer, TlsServerError};

/// This is a simplified, blocking/synchronous socket address resolver
pub fn resolve_address(host: &str, port: u16) -> Result<SocketAddr> {
    let addr = (host, port)
        .to_socket_addrs()?
        .next()
        .ok_or_else(|| TlsClientError::ResolutionFailure(String::from(host), port))?;
    Ok(addr)
}
