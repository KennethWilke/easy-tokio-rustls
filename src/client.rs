use anyhow::Result;
use std::convert::TryFrom;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio_rustls::rustls::{self, ClientConfig, RootCertStore};
use tokio_rustls::TlsConnector;

use crate::{client, TlsStream};
use crate::resolve_address;
use crate::{certificates, EasyTlsError};

fn get_client_config(root_store: RootCertStore) -> ClientConfig {
    rustls::ClientConfig::builder()
        .with_safe_defaults()
        .with_root_certificates(root_store)
        .with_no_client_auth()
}

/// Primary class for creating and connecting clientside TLS sockets
pub struct TlsClient {
    host: String,
    address: SocketAddr,
    cafile: Option<String>,
}

impl TlsClient {
    /// Returns a new [`TlsClient`] struct, address resolution will occur at
    /// creation. Connection attempts are performed later with calls to
    /// [`TlsClient::connect`]
    pub async fn new<T>(host: T) -> Result<Self>
    where
        T: ToString,
    {
        let host = host.to_string();
        let address = resolve_address(host.as_str()).await?;

        let host = match host.to_string().split_once(":") {
            Some((hostname, _)) => hostname.to_string(),
            None => return Err(EasyTlsError::ResolutionFailure(host).into()),
        };

        Ok(TlsClient {
            host,
            address,
            cafile: None,
        })
    }

    /// This function can be called if you need to set or use a certificate
    /// authority file instead of the defaults
    pub async fn set_cafile(&mut self, cafile: String) {
        self.cafile = Some(cafile)
    }

    /// This function triggers the actual connection to the remote TLS endpoint
    /// A [`TlsStream<TcpStream>`] handle will be returned on success
    pub async fn connect(&self) -> Result<TlsStream<TcpStream>> {
        let cafile = &self.cafile;
        let root_store = certificates::get_root_store(cafile.clone()).await?;
        let config = client::get_client_config(root_store);
        let connector = TlsConnector::from(Arc::new(config));
        let tcp = TcpStream::connect(self.address).await?;
        let domain = rustls::ServerName::try_from(self.host.as_str())?;
        let stream = connector.connect(domain, tcp).await?;
        Ok(stream.into())
    }
}
