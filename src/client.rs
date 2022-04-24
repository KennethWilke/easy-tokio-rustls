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

#[derive(Error, Debug)]
pub enum TlsClientError {
    #[error("Address resolution error: {0}:{1}")]
    ResolutionFailure(String, u16),
}

pub struct TlsClient {
    host: String,
    address: SocketAddr,
    cafile: Option<PathBuf>,
}

impl TlsClient {
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

    pub async fn set_cafile(&mut self, cafile: PathBuf) {
        self.cafile = Some(cafile)
    }

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
