//! Registry client abstraction — typed seam between `ship-store`'s install
//! pipeline and the cloud gateway's `RegistryService` + signed CDN.
//!
//! Two methods only: `resolve_version` (constraint → concrete version + the
//! registry-stated content hash) and `fetch_bundle` (path + version → the
//! verified bundle bytes + publisher attestation). The HTTP details +
//! attestation verification live in the concrete [`HttpRegistry`]; tests
//! substitute a closure / mock implementing the [`Registry`] trait.
//!
//! The attestation is NOT verified inside this trait — `extract_and_verify_bundle`
//! (in [`crate::install_bundle`]) is the verify gate. This trait is just the
//! transport: download the tarball, hand it back with the JWT. The install
//! pipeline holds the trusted keys and verifies.

use anyhow::{Context, Result};
use async_trait::async_trait;
use ship_gateway_client::{FetchBundleRequest, ResolveVersionRequest};

/// A resolved package version + the registry-stated content hash (the
/// canonical tree hash the attestation binds). The hash is opaque to this
/// trait — the install pipeline checks it against the extracted tree.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegistryVersion {
    pub version: String,
    pub sha256: String,
}

/// A fetched bundle: the gzip-tarball `bytes`, the publisher attestation JWT,
/// and the registry-stated content hash (must match the attestation's claim
/// AND the extracted tree hash).
#[derive(Debug, Clone)]
pub struct FetchedBundle {
    pub bytes: Vec<u8>,
    pub attestation_jwt: String,
    pub sha256: String,
}

/// Trait seam for the registry client — keeps `install_stack` testable
/// without standing up a Connect-RPC server.
#[async_trait]
pub trait Registry: Send + Sync {
    /// Resolve a semver constraint to a concrete published version.
    async fn resolve_version(&self, path: &str, constraint: &str) -> Result<RegistryVersion>;

    /// Fetch a bundle: the CDN-downloaded tarball bytes + the publisher
    /// attestation + the registry-stated content hash.
    async fn fetch_bundle(&self, path: &str, version: &str) -> Result<FetchedBundle>;
}

/// HTTP-backed [`Registry`] — drives the gateway `RegistryService` over
/// Connect-RPC and downloads the bundle from the signed CDN URL.
///
/// Generic over a `BundleFetcher` so the bundle download can be mocked
/// without touching `reqwest` from tests.
pub struct HttpRegistry<F: BundleFetcher = ReqwestBundleFetcher> {
    client: ship_gateway_client::RegistryServiceClient<ship_gateway_client::HttpClient>,
    fetcher: F,
}

impl HttpRegistry<ReqwestBundleFetcher> {
    /// Build the HTTP-backed registry client pointing at `gateway_base_url`.
    pub fn new(gateway_base_url: &str) -> Result<Self> {
        let client = ship_gateway_client::registry_client(gateway_base_url)
            .context("building gateway registry client")?;
        Ok(Self {
            client,
            fetcher: ReqwestBundleFetcher,
        })
    }
}

impl<F: BundleFetcher> HttpRegistry<F> {
    /// Build a registry with a custom bundle fetcher (test seam).
    pub fn with_fetcher(gateway_base_url: &str, fetcher: F) -> Result<Self> {
        let client = ship_gateway_client::registry_client(gateway_base_url)
            .context("building gateway registry client")?;
        Ok(Self { client, fetcher })
    }
}

#[async_trait]
impl<F: BundleFetcher> Registry for HttpRegistry<F> {
    async fn resolve_version(&self, path: &str, constraint: &str) -> Result<RegistryVersion> {
        let mut req = ResolveVersionRequest::default();
        req.path = path.to_string();
        req.constraint = constraint.to_string();
        let resp = self
            .client
            .resolve_version(req)
            .await
            .map_err(|e| anyhow::anyhow!("registry.resolve_version failed: {e}"))?;
        let view = resp.into_owned();
        Ok(RegistryVersion {
            version: view.version,
            sha256: view.sha256,
        })
    }

    async fn fetch_bundle(&self, path: &str, version: &str) -> Result<FetchedBundle> {
        let mut req = FetchBundleRequest::default();
        req.path = path.to_string();
        req.version = version.to_string();
        let resp = self
            .client
            .fetch_bundle(req)
            .await
            .map_err(|e| anyhow::anyhow!("registry.fetch_bundle failed: {e}"))?;
        let view = resp.into_owned();
        let bytes = self
            .fetcher
            .fetch(&view.download_url)
            .await
            .context("downloading bundle from CDN")?;
        Ok(FetchedBundle {
            bytes,
            attestation_jwt: view.attestation_jwt,
            sha256: view.sha256,
        })
    }
}

/// Bundle fetcher — downloads a tarball from a signed CDN URL. Separated so
/// tests can substitute an in-memory bundle without going through `reqwest`.
#[async_trait]
pub trait BundleFetcher: Send + Sync {
    async fn fetch(&self, url: &str) -> Result<Vec<u8>>;
}

/// Default fetcher using `reqwest::get`. The CDN URL is the publisher-signed
/// URL the gateway returned; TLS terminates at the CDN's edge.
pub struct ReqwestBundleFetcher;

#[async_trait]
impl BundleFetcher for ReqwestBundleFetcher {
    async fn fetch(&self, url: &str) -> Result<Vec<u8>> {
        let resp = reqwest::get(url)
            .await
            .with_context(|| format!("GET {url}"))?;
        if !resp.status().is_success() {
            anyhow::bail!("bundle download returned {}", resp.status());
        }
        // Early reject when the server advertises an oversized body.
        if let Some(declared) = resp.content_length()
            && (declared as u64) > bundle_limits::MAX_BUNDLE_DOWNLOAD
        {
            anyhow::bail!(
                "bundle download from {url} declared {declared} bytes, \
                 exceeds MAX_BUNDLE_DOWNLOAD ({} bytes)",
                bundle_limits::MAX_BUNDLE_DOWNLOAD
            );
        }
        // Cap the streaming read so a hostile or oversized body cannot
        // OOM the daemon. content-length is a hint, not a guarantee.
        let mut out = Vec::new();
        let mut stream = resp;
        let mut total: u64 = 0;
        while let Some(chunk) = stream
            .chunk()
            .await
            .with_context(|| format!("reading bundle chunk from {url}"))?
        {
            total = total.saturating_add(chunk.len() as u64);
            if total > bundle_limits::MAX_BUNDLE_DOWNLOAD {
                anyhow::bail!(
                    "bundle download from {url} exceeded MAX_BUNDLE_DOWNLOAD ({} bytes)",
                    bundle_limits::MAX_BUNDLE_DOWNLOAD
                );
            }
            out.extend_from_slice(&chunk);
        }
        Ok(out)
    }
}
