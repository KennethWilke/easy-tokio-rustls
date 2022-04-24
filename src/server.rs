use std::{fs::File, io::BufReader, net::SocketAddr, sync::Arc};

use anyhow::Result;
use rustls_pemfile::{certs, rsa_private_keys};
use thiserror::Error;
use tokio::net::{TcpListener, TcpStream};
use tokio_rustls::{
    rustls::{self, Certificate, PrivateKey, ServerConfig},
    TlsAcceptor, TlsStream,
};

use crate::resolve_address;

#[derive(Error, Debug)]
pub enum TlsServerError {
    #[error("Address resolution error: {0}:{1}")]
    ResolutionFailure(String, u16),
}

pub struct TlsServer {
    config: Arc<ServerConfig>,
    address: SocketAddr,
    pub interface: String,
}

impl TlsServer {
    pub async fn new<T, U, V>(interface: T, port: u16, cert_file: U, key_file: V) -> Result<Self>
    where
        T: ToString,
        U: ToString,
        V: ToString,
    {
        let interface = interface.to_string();
        let cert_file = cert_file.to_string();
        let key_file = key_file.to_string();

        let certificates = load_certificates(cert_file)?;
        let mut keys = load_keys(key_file)?;
        println!("keys: {}", keys.len());
        let address = resolve_address(interface.as_str(), port)?;

        let config = rustls::ServerConfig::builder()
            .with_safe_defaults()
            .with_no_client_auth()
            .with_single_cert(certificates, keys.remove(0))?;

        Ok(TlsServer {
            config: Arc::new(config),
            address,
            interface: interface.into(),
        })
    }

    pub async fn listen(&self) -> Result<TlsListener> {
        Ok(TlsListener::new(&self.address, &self.config).await?)
    }
}

pub struct TlsListener {
    listener: TcpListener,
    acceptor: TlsAcceptor,
}

impl TlsListener {
    pub async fn new(address: &SocketAddr, config: &Arc<ServerConfig>) -> Result<Self> {
        let acceptor = TlsAcceptor::from(config.clone());
        let listener = TcpListener::bind(address).await?;
        Ok(TlsListener { listener, acceptor })
    }

    pub async fn socket_accept(&self) -> Result<(TcpClientStream, SocketAddr)> {
        let (stream, address) = self.listener.accept().await?;
        let acceptor = self.acceptor.clone();
        Ok((TcpClientStream { stream, acceptor }, address))
    }
}

pub struct TcpClientStream {
    stream: TcpStream,
    acceptor: TlsAcceptor,
}

impl TcpClientStream {
    pub async fn tls_accept(self) -> Result<TlsStream<TcpStream>> {
        let stream = self.acceptor.accept(self.stream).await?;
        let stream = tokio_rustls::TlsStream::Server(stream);
        Ok(stream)
    }
}

// TODO make async!
fn load_certificates(path: String) -> Result<Vec<Certificate>> {
    let mut reader = BufReader::new(File::open(path)?);
    let mut certificates: Vec<Certificate> = Vec::new();
    for cert in certs(&mut reader)? {
        certificates.push(Certificate(cert))
    }
    Ok(certificates)
}

// TODO make async!
fn load_keys(path: String) -> Result<Vec<PrivateKey>> {
    let mut reader = BufReader::new(File::open(path)?);
    let mut keys: Vec<PrivateKey> = Vec::new();
    for key in rsa_private_keys(&mut reader)? {
        keys.push(PrivateKey(key))
    }
    Ok(keys)
}
