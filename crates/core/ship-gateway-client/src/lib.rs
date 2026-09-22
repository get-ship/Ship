//! Typed Connect-RPC client for the Ship gateway.
//!
//! Proto bindings are generated once in the published `ship-proto` crate
//! (ship-contracts SSOT) and re-exported here; this crate adds the
//! hand-written TLS transport + `auth_client()` helper. The cloud gateway is
//! the corresponding TS server, generated from the same protos via buf —
//! single source of truth, two consumers.
//!
//! ## Quick start
//!
//! ```no_run
//! use ship_gateway_client::{auth_client, TokenExchangeRequest};
//! # async fn demo() -> Result<(), Box<dyn std::error::Error>> {
//! let client = auth_client("https://api.getship.dev")?;
//! let resp = client
//!     .token_exchange(TokenExchangeRequest {
//!         code: "auth-code".into(),
//!         verifier: "pkce-verifier".into(),
//!         ..Default::default()
//!     })
//!     .await?;
//! println!("got key: {}", resp.view().key);
//! # Ok(())
//! # }
//! ```
//!
//! Default gateway base URL convention: `https://api.getship.dev` (prod)
//! or `https://staging.getship.dev` (staging). Override via
//! [`auth_client()`] argument; the transport is the same.

pub use connectrpc::client::{CallOptions, ClientConfig, HttpClient};
pub use connectrpc::ConnectError;

// Generated bindings live in the published `ship-proto` crate (ship-contracts
// SSOT); re-export the gateway tree so consumers keep importing
// `ship_gateway_client::ship::gateway::v1::*` and the crate-root glob unchanged.
pub mod ship {
    pub use ship_proto::ship::gateway;
}

pub use ship_proto::ship::gateway::v1::*;

/// Errors building a gateway client.
#[derive(Debug, thiserror::Error)]
pub enum BuildError {
    #[error("invalid base URL: {0}")]
    InvalidUri(#[from] http::uri::InvalidUri),
    #[error("loading system root certificates failed: {0}")]
    TlsRoots(std::io::Error),
    #[error("rustls config build failed: {0}")]
    Tls(rustls::Error),
}

/// Build a TLS-enabled [`AuthServiceClient`] pointed at `base_url`.
/// Default codec is Connect protocol with protobuf binary; override via
/// `client.config_mut()` if a caller needs JSON or gRPC framing.
///
/// **TLS roots:** loaded from the system trust store via
/// `rustls-native-certs`. This means the client trusts whatever the host
/// OS trusts — appropriate for production where users have correctly
/// configured machines, but worth noting if a future feature lets users
/// install custom CAs (e.g., enterprise MITM proxies).
pub fn auth_client(base_url: &str) -> Result<AuthServiceClient<HttpClient>, BuildError> {
    let uri: http::Uri = base_url.parse()?;
    let transport = build_https_transport()?;
    Ok(AuthServiceClient::new(transport, ClientConfig::new(uri)))
}

/// Build a TLS-enabled [`RegistryServiceClient`] pointed at `base_url`.
///
/// The registry service is the typed Connect surface for the bundle registry
/// (`ResolveVersion` + `FetchBundle`); the install pipeline consumes this
/// client. Same TLS posture as [`auth_client`] — system roots, ring crypto
/// provider auto-installed on first TLS object build.
pub fn registry_client(base_url: &str) -> Result<RegistryServiceClient<HttpClient>, BuildError> {
    let uri: http::Uri = base_url.parse()?;
    let transport = build_https_transport()?;
    Ok(RegistryServiceClient::new(transport, ClientConfig::new(uri)))
}

fn build_https_transport() -> Result<HttpClient, BuildError> {
    // rustls 0.23 requires a `CryptoProvider` to be installed at the
    // process level before TLS objects can be built. Installing `ring`
    // is idempotent — second and later calls are no-ops — so safe for
    // libraries to do without coordination across consumers.
    let _ = rustls::crypto::ring::default_provider().install_default();

    let mut roots = rustls::RootCertStore::empty();
    let native = rustls_native_certs::load_native_certs();
    if !native.errors.is_empty() {
        // Surface the first error but proceed with whatever certs loaded;
        // partial trust store is the standard rustls-native-certs behavior.
        eprintln!(
            "ship-gateway-client: warning loading native certs ({} errors); proceeding with what loaded",
            native.errors.len()
        );
    }
    for cert in native.certs {
        let _ = roots.add(cert);
    }
    let cfg = rustls::ClientConfig::builder()
        .with_root_certificates(roots)
        .with_no_client_auth();
    Ok(HttpClient::with_tls(std::sync::Arc::new(cfg)))
}
