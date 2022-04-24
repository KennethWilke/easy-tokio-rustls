use anyhow::Result;
use std::convert::TryFrom;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use thiserror::Error;
use tokio::net::TcpStream;
use tokio_rustls::client::TlsStream;
use tokio_rustls::rustls::{self, ClientConfig, RootCertStore};
use tokio_rustls::TlsConnector;

use crate::certstore;
use crate::client;
use crate::resolve_address;

fn get_client_config(root_store: RootCertStore) -> ClientConfig {
    rustls::ClientConfig::builder()
        .with_safe_defaults()
        .with_root_certificates(root_store)
        .with_no_client_auth()
}

/// Represents errors that may be returned through [`TlsClient`] usage.
#[derive(Error, Debug)]
pub enum TlsClientError {
    #[error("Address resolution error: {0}:{1}")]
    ResolutionFailure(String, u16),
}

/// Primary class for creating and connecting clientside TLS sockets
pub struct TlsClient {
    host: String,
    address: SocketAddr,
    cafile: Option<PathBuf>,
}

impl TlsClient {
    /// Returns a new [`TlsClient`] struct, this currently using the blocking
    /// address resolution method upon creation. In future changes I will make
    /// this use asynchronous resolution, but resolution will still occur at
    /// creation. Connection attempts are performed later with calls to
    /// [`TlsClient::connect`]
    pub async fn new<T>(host: T, port: u16) -> Result<Self>
    where
        T: ToString,
    {
        let host = host.to_string();
        let address = resolve_address(host.as_str(), port)?;

        Ok(TlsClient {
            host,
            address,
            cafile: None,
        })
    }

    /// This function can be called if you need to set or use a certificate
    /// authority file instead of the defaults
    pub async fn set_cafile(&mut self, cafile: PathBuf) {
        self.cafile = Some(cafile)
    }

    /// This function triggers the actual connection to the remote TLS endpoint
    /// A [`TlsStream<TcpStream>`] handle will be returned on success
    pub async fn connect(&self) -> Result<TlsStream<TcpStream>> {
        let cafile = &self.cafile;
        let root_store = certstore::get_root_store(cafile)?;
        let config = client::get_client_config(root_store);
        let connector = TlsConnector::from(Arc::new(config));
        let tcp = TcpStream::connect(self.address).await?;
        let domain = rustls::ServerName::try_from(self.host.as_str())?;
        Ok(connector.connect(domain, tcp).await?)
    }
}
