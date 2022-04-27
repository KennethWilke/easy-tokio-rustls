//! This library provides convenient abstractions for creating simple TLS
//! sockets with `tokio-rustls`. Examples can be found [in the examples directory](https://github.com/KennethWilke/easy-tokio-rustls/tree/main/examples)

use anyhow::Result;
use std::net::SocketAddr;
use thiserror::Error;
use tokio::net::{lookup_host, ToSocketAddrs};

mod certificates;
mod client;
mod server;

pub use client::TlsClient;
pub use server::{TlsServer,TlsListener};
pub use tokio_rustls::TlsStream;

/// Represents custom errors returned directly by this crate
#[derive(Error, Debug)]
pub enum EasyTlsError {
    /// Returned for address resolution failures
    #[error("Failed to resolve address for '{0}'")]
    ResolutionFailure(String),

    /// Returned when reading certificates or keys fails
    #[error("Failed to resolve address for '{0}'")]
    CertificateError(String),
}

/// This is a simplified async address resolver
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
