//! TLS para PostgreSQL via rustls (provider ring: sin OpenSSL ni cmake/NASM).
//!
//! Paridad con libpq/psycopg2: en sslmode `require`/`prefer`/`allow` NO se
//! verifica el certificado del servidor (asi funcionan hoy los hubs con
//! certificados self-signed o managed); `verify-ca`/`verify-full` si
//! verifican contra los certificados nativos del sistema.

use std::sync::Arc;

use anyhow::Context;
use rustls::client::danger::{
    HandshakeSignatureValid, ServerCertVerified, ServerCertVerifier,
};
use rustls::crypto::CryptoProvider;
use rustls::pki_types::{CertificateDer, ServerName, UnixTime};
use rustls::{ClientConfig, DigitallySignedStruct, SignatureScheme};
use tokio_postgres_rustls::MakeRustlsConnect;

/// Verificador que acepta cualquier certificado (semantica sslmode=require).
#[derive(Debug)]
struct NoVerifier(Arc<CryptoProvider>);

impl ServerCertVerifier for NoVerifier {
    fn verify_server_cert(
        &self,
        _end_entity: &CertificateDer<'_>,
        _intermediates: &[CertificateDer<'_>],
        _server_name: &ServerName<'_>,
        _ocsp_response: &[u8],
        _now: UnixTime,
    ) -> Result<ServerCertVerified, rustls::Error> {
        Ok(ServerCertVerified::assertion())
    }

    fn verify_tls12_signature(
        &self,
        message: &[u8],
        cert: &CertificateDer<'_>,
        dss: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, rustls::Error> {
        rustls::crypto::verify_tls12_signature(
            message,
            cert,
            dss,
            &self.0.signature_verification_algorithms,
        )
    }

    fn verify_tls13_signature(
        &self,
        message: &[u8],
        cert: &CertificateDer<'_>,
        dss: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, rustls::Error> {
        rustls::crypto::verify_tls13_signature(
            message,
            cert,
            dss,
            &self.0.signature_verification_algorithms,
        )
    }

    fn supported_verify_schemes(&self) -> Vec<SignatureScheme> {
        self.0
            .signature_verification_algorithms
            .supported_schemes()
    }
}

/// Construye el conector TLS segun DB_SSL_MODE.
pub fn make_connector(sslmode: &str) -> anyhow::Result<MakeRustlsConnect> {
    let provider = Arc::new(rustls::crypto::ring::default_provider());
    let builder = ClientConfig::builder_with_provider(provider.clone())
        .with_safe_default_protocol_versions()
        .context("rustls: no se pudieron fijar versiones de protocolo")?;
    let config = match sslmode {
        "verify-ca" | "verify-full" => {
            let mut roots = rustls::RootCertStore::empty();
            let certs = rustls_native_certs::load_native_certs();
            for cert in certs.certs {
                let _ = roots.add(cert);
            }
            builder
                .with_root_certificates(roots)
                .with_no_client_auth()
        }
        _ => builder
            .dangerous()
            .with_custom_certificate_verifier(Arc::new(NoVerifier(provider)))
            .with_no_client_auth(),
    };
    Ok(MakeRustlsConnect::new(config))
}
