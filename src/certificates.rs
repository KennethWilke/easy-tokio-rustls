use anyhow::Result;
use regex::Regex;
use tokio::io::AsyncReadExt;
use tokio_rustls::rustls::{self, OwnedTrustAnchor, RootCertStore};
use tokio_rustls::webpki;

use tokio_rustls::rustls::{Certificate, PrivateKey};

const MAX_PEM_SIZE: usize = 32 * 1024;
const PEM_REGEX: &str = r"-----BEGIN (.+?)-----\n(?s)(.+?)\n-----END (.+?)-----";

/// Loads the root certificate storage, uses defaults if no cafile
pub async fn get_root_store(cafile: Option<String>) -> Result<RootCertStore> {
    let cert_store = match cafile {
        Some(cafile) => get_cafile_store(cafile).await,
        None => get_default_store(),
    }?;
    Ok(cert_store)
}

async fn get_cafile_store(cafile: String) -> Result<RootCertStore> {
    let mut cert_store = rustls::RootCertStore::empty();
    let certs = read_pem(cafile).await?;
    let trust_anchors = certs.iter().map(|cert| {
        let trust_anchor = webpki::TrustAnchor::try_from_cert_der(&cert[..]).unwrap();
        OwnedTrustAnchor::from_subject_spki_name_constraints(
            trust_anchor.subject,
            trust_anchor.spki,
            trust_anchor.name_constraints,
        )
    });
    cert_store.add_server_trust_anchors(trust_anchors);
    Ok(cert_store)
}

fn get_default_store() -> Result<RootCertStore> {
    let mut cert_store = rustls::RootCertStore::empty();
    cert_store.add_server_trust_anchors(webpki_roots::TLS_SERVER_ROOTS.0.iter().map(
        |trust_anchor| {
            OwnedTrustAnchor::from_subject_spki_name_constraints(
                trust_anchor.subject,
                trust_anchor.spki,
                trust_anchor.name_constraints,
            )
        },
    ));
    Ok(cert_store)
}

/// Loads certificates from the given file
pub async fn load_certificates(path: String) -> Result<Vec<Certificate>> {
    let mut certificates: Vec<Certificate> = Vec::new();
    for cert in read_pem(path).await? {
        certificates.push(Certificate(cert))
    }
    Ok(certificates)
}

/// Loads private keys from the given file
pub async fn load_keys(path: String) -> Result<Vec<PrivateKey>> {
    let mut keys: Vec<PrivateKey> = Vec::new();
    for key in read_pem(path).await? {
        keys.push(PrivateKey(key))
    }
    Ok(keys)
}

/// Loads PEM encoded entities
pub async fn read_pem(path: String) -> Result<Vec<Vec<u8>>> {
    let mut file = tokio::fs::File::open(path).await?;
    let mut buffer = [0; MAX_PEM_SIZE];
    let size = file.read(&mut buffer).await?;
    let pem_text = std::str::from_utf8(&buffer[..size])?;
    let mut results = Vec::new();

    let re = Regex::new(PEM_REGEX)?;
    for capture in re.captures_iter(pem_text) {
        let header = capture.get(1).unwrap().as_str();
        let encoded = capture.get(2).unwrap().as_str().replace("\n", "");
        let footer = capture.get(3).unwrap().as_str();
        if header != footer {
            panic!("PEM header doesn't match footer")
        }

        results.push(base64::decode(encoded.as_bytes())?.to_vec());
    }

    Ok(results)
}
