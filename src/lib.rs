//! This library provides convenient abstractions for creating simple TLS
//! sockets with `tokio-rustls`.

use anyhow::Result;
use std::net::SocketAddr;
use thiserror::Error;
use tokio::net::{lookup_host, ToSocketAddrs};

mod certificates;
mod client;
mod server;

pub use client::TlsClient;
pub use server::TlsServer;

/// Represents custom errors returned directly by this crate
#[derive(Error, Debug)]
pub enum EasyTlsError {
    #[error("Failed to resolve address for '{0}'")]
    ResolutionFailure(String),
}

/// This is a simplified address resolver
pub async fn resolve_address<T>(host: T) -> Result<SocketAddr>
where
    T: ToSocketAddrs + ToString + Copy,
{
    let mut addresses = lookup_host(host).await?;
    let address = addresses
        .next()
        .ok_or_else(|| EasyTlsError::ResolutionFailure(host.to_string()))?;
    Ok(address)
}
