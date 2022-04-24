use anyhow::Result;
use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use tokio_rustls::rustls::{self, OwnedTrustAnchor, RootCertStore};
use tokio_rustls::webpki;

use rustls_pemfile::{certs, rsa_private_keys};

use tokio_rustls::rustls::{Certificate, PrivateKey};

pub fn get_root_store(cafile: &Option<PathBuf>) -> Result<RootCertStore> {
    let cert_store = match cafile {
        Some(cafile) => get_cafile_store(cafile),
        None => get_default_store(),
    }?;
    Ok(cert_store)
}

fn get_cafile_store(cafile: &Path) -> Result<RootCertStore> {
    let mut cert_store = rustls::RootCertStore::empty();
    let mut pemfile = BufReader::new(File::open(cafile)?);
    let certs = rustls_pemfile::certs(&mut pemfile)?;
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

// TODO make async!
pub fn load_certificates(path: String) -> Result<Vec<Certificate>> {
    let mut reader = BufReader::new(File::open(path)?);
    let mut certificates: Vec<Certificate> = Vec::new();
    for cert in certs(&mut reader)? {
        certificates.push(Certificate(cert))
    }
    Ok(certificates)
}

// TODO make async!
pub fn load_keys(path: String) -> Result<Vec<PrivateKey>> {
    let mut reader = BufReader::new(File::open(path)?);
    let mut keys: Vec<PrivateKey> = Vec::new();
    for key in rsa_private_keys(&mut reader)? {
        keys.push(PrivateKey(key))
    }
    Ok(keys)
}
