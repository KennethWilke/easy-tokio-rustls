use anyhow::Result;
use std::net::{SocketAddr, ToSocketAddrs};

mod certstore;
mod client;

pub use client::{TlsClient, TlsClientError};

pub fn resolve_address(host: &str, port: u16) -> Result<SocketAddr> {
    let addr = (host, port)
        .to_socket_addrs()?
        .next()
        .ok_or(TlsClientError::ResolutionFailure(String::from(host), port))?;
    Ok(addr)
}
