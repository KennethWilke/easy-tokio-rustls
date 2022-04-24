use anyhow::Result;
use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use tokio_rustls::rustls::{self, OwnedTrustAnchor, RootCertStore};
use tokio_rustls::webpki;

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
