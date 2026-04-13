//! Server-side attestation — daemon accepts a JWT signed by a trusted issuer
//! (getship.dev) and issues a session token without CLI pairing.
//!
//! The daemon verifies the JWT against a JWKS fetched from the configured
//! issuer. Cached JWKS with 1h TTL; re-fetched on signature failure.

use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

use jsonwebtoken::{Algorithm, DecodingKey, Validation, decode, decode_header};
use serde::{Deserialize, Serialize};

/// Default issuer/JWKS URL. Overridden via ~/.ship/config.toml [auth]
/// attestation_issuer.
pub const DEFAULT_ATTESTATION_ISSUER: &str = "https://getship.dev";

/// How long to cache the fetched JWKS.
const JWKS_CACHE_TTL: Duration = Duration::from_secs(3600);

/// Claims we expect in a daemon-attestation JWT. Extra claims are ignored.
#[derive(Debug, Deserialize, Serialize)]
pub struct AttestationClaims {
    pub iss: String,
    pub sub: String,
    pub aud: String,
    pub iat: i64,
    pub exp: i64,
    #[serde(default)]
    pub plan: Option<String>,
    #[serde(default)]
    pub caps: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct AttestationConfig {
    pub issuer: String,
    pub jwks_url: String,
}

impl AttestationConfig {
    /// Build from a base issuer URL. JWKS URL is `{issuer}/api/jwks`.
    pub fn from_issuer(issuer: impl Into<String>) -> Self {
        let issuer = issuer.into();
        let jwks_url = format!("{}/api/jwks", issuer.trim_end_matches('/'));
        Self { issuer, jwks_url }
    }
}

/// JWKS JSON shape (subset).
#[derive(Debug, Clone, Deserialize)]
struct Jwks {
    keys: Vec<Jwk>,
}

#[derive(Debug, Clone, Deserialize)]
struct Jwk {
    kid: Option<String>,
    #[allow(dead_code)]
    alg: Option<String>,
    kty: String,
    #[serde(default)]
    crv: Option<String>,
    /// Ed25519 public key bytes, base64url-no-pad encoded.
    #[serde(default)]
    x: Option<String>,
}

struct CachedJwks {
    jwks: Jwks,
    fetched_at: Instant,
}

/// Shared JWKS cache used by server-attest handler.
#[derive(Default, Clone)]
pub struct AttestationVerifier {
    cache: Arc<Mutex<Option<CachedJwks>>>,
}

impl AttestationVerifier {
    pub fn new() -> Self {
        Self::default()
    }

    /// Verify `jwt` against JWKS fetched from `config.jwks_url`. On signature
    /// failure, refetches JWKS once before giving up — handles mid-rotation.
    pub async fn verify(
        &self,
        jwt: &str,
        config: &AttestationConfig,
    ) -> Result<AttestationClaims, VerifyError> {
        // First attempt with cached JWKS.
        match self.verify_with_cache(jwt, config, false).await {
            Ok(claims) => Ok(claims),
            Err(VerifyError::SignatureInvalid) | Err(VerifyError::KeyNotFound) => {
                // Could be mid-rotation. Force refetch and retry.
                self.verify_with_cache(jwt, config, true).await
            }
            Err(e) => Err(e),
        }
    }

    async fn verify_with_cache(
        &self,
        jwt: &str,
        config: &AttestationConfig,
        force_refresh: bool,
    ) -> Result<AttestationClaims, VerifyError> {
        let jwks = self.load_jwks(&config.jwks_url, force_refresh).await?;
        let header = decode_header(jwt).map_err(|_| VerifyError::Malformed)?;

        // Find matching key by kid, or first EdDSA key.
        let jwk = match &header.kid {
            Some(kid) => jwks.keys.iter().find(|k| k.kid.as_deref() == Some(kid.as_str())),
            None => jwks.keys.first(),
        }
        .ok_or(VerifyError::KeyNotFound)?;

        let key_material = jwk.x.as_deref().ok_or(VerifyError::UnsupportedKey)?;
        if jwk.kty != "OKP" || jwk.crv.as_deref() != Some("Ed25519") {
            return Err(VerifyError::UnsupportedKey);
        }

        let decoding_key = DecodingKey::from_ed_components(key_material)
            .map_err(|_| VerifyError::UnsupportedKey)?;

        let mut validation = Validation::new(Algorithm::EdDSA);
        validation.set_audience(&["ship-daemon"]);
        validation.set_issuer(&[&config.issuer]);

        let token = decode::<AttestationClaims>(jwt, &decoding_key, &validation)
            .map_err(|e| match e.kind() {
                jsonwebtoken::errors::ErrorKind::InvalidSignature => VerifyError::SignatureInvalid,
                jsonwebtoken::errors::ErrorKind::ExpiredSignature => VerifyError::Expired,
                jsonwebtoken::errors::ErrorKind::InvalidAudience => VerifyError::WrongAudience,
                jsonwebtoken::errors::ErrorKind::InvalidIssuer => VerifyError::WrongIssuer,
                _ => VerifyError::Malformed,
            })?;

        Ok(token.claims)
    }

    async fn load_jwks(&self, url: &str, force_refresh: bool) -> Result<Jwks, VerifyError> {
        if !force_refresh {
            let cache = self.cache.lock().await;
            if let Some(cached) = cache.as_ref() {
                if cached.fetched_at.elapsed() < JWKS_CACHE_TTL {
                    return Ok(cached.jwks.clone());
                }
            }
        }
        drop(self.cache.lock().await); // release before network I/O

        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .map_err(|_| VerifyError::JwksUnavailable)?;
        let jwks: Jwks = client
            .get(url)
            .send()
            .await
            .map_err(|_| VerifyError::JwksUnavailable)?
            .json()
            .await
            .map_err(|_| VerifyError::JwksMalformed)?;

        let mut cache = self.cache.lock().await;
        *cache = Some(CachedJwks {
            jwks: jwks.clone(),
            fetched_at: Instant::now(),
        });
        Ok(jwks)
    }
}

#[derive(Debug, PartialEq)]
pub enum VerifyError {
    Malformed,
    SignatureInvalid,
    Expired,
    WrongAudience,
    WrongIssuer,
    KeyNotFound,
    UnsupportedKey,
    JwksUnavailable,
    JwksMalformed,
}

impl std::fmt::Display for VerifyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VerifyError::Malformed => write!(f, "jwt malformed"),
            VerifyError::SignatureInvalid => write!(f, "signature invalid"),
            VerifyError::Expired => write!(f, "token expired"),
            VerifyError::WrongAudience => write!(f, "audience mismatch"),
            VerifyError::WrongIssuer => write!(f, "issuer mismatch"),
            VerifyError::KeyNotFound => write!(f, "signing key not found in JWKS"),
            VerifyError::UnsupportedKey => write!(f, "unsupported key type (require Ed25519)"),
            VerifyError::JwksUnavailable => write!(f, "could not reach JWKS endpoint"),
            VerifyError::JwksMalformed => write!(f, "JWKS response malformed"),
        }
    }
}

/// KV namespace for the daemon-owner user ID. Set by `ship login`, checked
/// on every attestation.
pub const DAEMON_OWNER_NAMESPACE: &str = "auth.daemon_owner";
pub const DAEMON_OWNER_KEY: &str = "user_id";

/// Return the user ID that owns this daemon, or None if not yet bound.
pub fn daemon_owner() -> Option<String> {
    runtime::db::kv::get(DAEMON_OWNER_NAMESPACE, DAEMON_OWNER_KEY)
        .ok()
        .flatten()
        .and_then(|v| v.as_str().map(String::from))
}

/// Bind this daemon to a specific user ID. Called by `ship login`.
pub fn set_daemon_owner(user_id: &str) -> anyhow::Result<()> {
    let value = serde_json::Value::String(user_id.to_string());
    runtime::db::kv::set(DAEMON_OWNER_NAMESPACE, DAEMON_OWNER_KEY, &value)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_from_issuer_builds_jwks_url() {
        let cfg = AttestationConfig::from_issuer("https://getship.dev");
        assert_eq!(cfg.jwks_url, "https://getship.dev/api/jwks");

        let cfg = AttestationConfig::from_issuer("https://getship.dev/");
        assert_eq!(cfg.jwks_url, "https://getship.dev/api/jwks");
    }

    #[test]
    fn daemon_owner_roundtrip() {
        set_daemon_owner("user-abc-123").unwrap();
        assert_eq!(daemon_owner().as_deref(), Some("user-abc-123"));
    }
}
