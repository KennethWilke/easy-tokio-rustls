use std::{net::SocketAddr, sync::Arc};

use anyhow::Result;
use tokio::net::{TcpListener, TcpStream};
use tokio_rustls::{
    rustls::{self, ServerConfig},
    TlsAcceptor, TlsStream,
};

use crate::certificates::{load_certificates, load_keys};
use crate::resolve_address;

/// Primary class for creating and connecting serverside TLS sockets
pub struct TlsServer {
    config: Arc<ServerConfig>,
    address: SocketAddr,
    interface: String,
}

impl TlsServer {
    /// Creates a new [`TlsServer`] struct instance. This call will load
    /// certificates and private keys from the provided file paths, then
    /// resolve the interface/port combination provided. If this succeeds, a
    /// rustls [`ServerConfig`] structure is built internally that uses default
    /// settings using the first private key from the key_file. The address
    /// binding and listening occurs later when [`TlsServer::listen`] is
    /// called.
    pub async fn new<T, U, V>(interface: T, cert_file: U, key_file: V) -> Result<Self>
    where
        T: ToString,
        U: ToString,
        V: ToString,
    {
        let interface = interface.to_string();
        let cert_file = cert_file.to_string();
        let key_file = key_file.to_string();

        let certificates = load_certificates(cert_file).await?;
        let mut keys = load_keys(key_file).await?;

        let address = resolve_address(interface.as_str()).await?;

        let config = rustls::ServerConfig::builder()
            .with_safe_defaults()
            .with_no_client_auth()
            .with_single_cert(certificates, keys.remove(0))?;

        Ok(TlsServer {
            config: Arc::new(config),
            address,
            interface,
        })
    }

    /// Starts the listening side of the server socket, returning a [`TlsListener`]
    pub async fn listen(&self) -> Result<TlsListener> {
        Ok(TlsListener::new(&self.address, &self.config).await?)
    }

    /// Returns the interface string that was used during creation
    pub fn get_interface(&self) -> &String {
        &self.interface
    }
}

/// This structure is a handle to a listening socket returned by
/// [`TlsServer::listen`]. Call [`TlsListener::stream_accept`] to accept a new
/// client connection.
pub struct TlsListener {
    listener: TcpListener,
    acceptor: TlsAcceptor,
}

impl TlsListener {
    async fn new(address: &SocketAddr, config: &Arc<ServerConfig>) -> Result<Self> {
        let acceptor = TlsAcceptor::from(config.clone());
        let listener = TcpListener::bind(address).await?;
        Ok(TlsListener { listener, acceptor })
    }

    /// Call this method to accept the next client, this will return a
    /// [`TcpClientStream`], which represents a client connection that has not
    /// yet performed TLS negotiation
    pub async fn stream_accept(&self) -> Result<(TcpClientStream, SocketAddr)> {
        let (stream, address) = self.listener.accept().await?;
        let acceptor = self.acceptor.clone();
        Ok((TcpClientStream { stream, acceptor }, address))
    }
}

/// This is a client stream that has connected, but has not yet performed TLS
/// negotiation. Call [`TcpClientStream::tls_accept`] to engage TLS negotiation
pub struct TcpClientStream {
    stream: TcpStream,
    acceptor: TlsAcceptor,
}

impl TcpClientStream {
    /// Attempts to accept a TLS connection from a client. If successful, a
    /// [`TlsStream<TcpStream>`] socket handle is returned to use for encrypted
    /// communication.
    pub async fn tls_accept(self) -> Result<TlsStream<TcpStream>> {
        let stream = self.acceptor.accept(self.stream).await?;
        let stream = tokio_rustls::TlsStream::Server(stream);
        Ok(stream)
    }
}
