//! `AccountService` — orchestrates login / complete_login / state / logout /
//! get_key for the daemon's Ship Cloud identity.
//!
//! Owns: the token store (file-backed over substrate config_doc), the
//! in-memory pending-login registry, the gateway Connect client, and a
//! `KernelLog` handle for `account.*` audit. The owner-binding
//! (`daemon.owner.bound/unbound`) is an injected `OwnerBinder` so this crate
//! does not depend on the `ship-platform` parent; the daemon wires
//! `auth::attest::{set,clear}_daemon_owner` (or `KernelOwnerBinder`, below).
//!
//! PKCE flow split (unchanged from the legacy module): `start_login` registers
//! a verifier + returns the browser URL; the CLI's callback server gets the
//! code; `complete_login` exchanges code+verifier for the API key, stores it,
//! and binds the owner.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

use base64::Engine as _;
use sha2::{Digest, Sha256};
use tokio::sync::Mutex;

use ship_gateway_client::{
    auth_client, AuthServiceClient, CallOptions, GetMeRequest, HttpClient, RevokeKeyRequest,
    TokenExchangeRequest,
};
use ship_substrate::ports::KernelLog;

use crate::error::AccountError;
use crate::events;
use crate::store::AccountTokenStore;
use crate::types::{AccountState, AccountToken, MeView};

/// In-memory record of a login that's been started but not yet exchanged. Lives
/// only on the daemon; dropped on restart (the user just retries `ship login`).
#[derive(Debug, Clone)]
struct Pending {
    code_verifier: String,
    created_at: Instant,
}

/// Maximum age for a pending login before GC. Five minutes covers a realistic
/// OAuth-in-browser duration; old entries would otherwise leak memory.
const PENDING_TTL_SECS: u64 = 5 * 60;

/// Owner-binding hook. The crate calls `bind` after a successful login and
/// `unbind` on logout. Injected so the crate stays decoupled from the
/// `ship-platform` parent's `auth::attest` (which depends on this crate).
#[async_trait::async_trait]
pub trait OwnerBinder: Send + Sync {
    async fn bind(&self, user_id: &str);
    async fn unbind(&self, user_id: &str);
}

/// Default `OwnerBinder` that emits `daemon.owner.bound/unbound` through the
/// `KernelLog` port (Account-scoped → `account.db`). Replaces the legacy
/// `auth::attest` `SqliteEventStore::append` bypass — same event types, same
/// single-row owner projection, but on the kernel path.
pub struct KernelOwnerBinder {
    kernel: Arc<KernelLog>,
}

impl KernelOwnerBinder {
    pub fn new(kernel: Arc<KernelLog>) -> Self {
        Self { kernel }
    }
}

#[async_trait::async_trait]
impl OwnerBinder for KernelOwnerBinder {
    async fn bind(&self, user_id: &str) {
        if let Err(e) = events::emit_owner_bound(&self.kernel, user_id).await {
            tracing::warn!("account: daemon-owner bind failed: {e}");
        }
    }
    async fn unbind(&self, user_id: &str) {
        if let Err(e) = events::emit_owner_unbound(&self.kernel, user_id).await {
            tracing::warn!("account: daemon-owner unbind failed: {e}");
        }
    }
}

/// Result of `start_login`.
#[derive(Debug, Clone)]
pub struct StartLoginResponse {
    /// Opaque id the client passes back in `complete_login`.
    pub state_id: String,
    /// URL the CLI opens in the browser; carries the PKCE challenge + state.
    pub browser_url: String,
}

pub struct AccountService {
    store: Arc<dyn AccountTokenStore>,
    pending: Arc<Mutex<HashMap<String, Pending>>>,
    /// Audit-event sink. Account-scoped events route to `account.db`.
    kernel: Arc<KernelLog>,
    /// Owner-binding hook (daemon.owner.*).
    owner: Arc<dyn OwnerBinder>,
    /// Gateway Connect client; built once, reused (pools connections).
    client: AuthServiceClient<HttpClient>,
    /// Base URL the client targets. Surfaced for diagnostics.
    base_url: String,
}

impl AccountService {
    /// Build a service over the substrate `KernelLog`, a token store, an
    /// owner-binder, and the gateway `base_url` (prod `https://api.getship.dev`).
    pub fn new(
        store: Arc<dyn AccountTokenStore>,
        kernel: Arc<KernelLog>,
        owner: Arc<dyn OwnerBinder>,
        base_url: impl Into<String>,
    ) -> Result<Self, AccountError> {
        let base_url = base_url.into();
        let client =
            auth_client(&base_url).map_err(|e| AccountError::Gateway(format!("building gateway client: {e}")))?;
        Ok(Self {
            store,
            pending: Arc::new(Mutex::new(HashMap::new())),
            kernel,
            owner,
            client,
            base_url,
        })
    }

    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    /// Start a login flow. Generates a PKCE verifier/challenge pair, registers a
    /// state id, returns the browser URL. `redirect_uri` must be the CLI's local
    /// callback; the gateway validates it is localhost-shaped.
    pub async fn start_login(&self, redirect_uri: &str) -> Result<StartLoginResponse, AccountError> {
        let verifier = generate_code_verifier();
        let challenge = s256_challenge(&verifier);
        let state_id = generate_state_id();

        self.gc_pending().await;
        self.pending.lock().await.insert(
            state_id.clone(),
            Pending { code_verifier: verifier, created_at: Instant::now() },
        );

        let browser_url = format!(
            "{}/v1/auth/cli?code_challenge={}&code_challenge_method=S256&redirect_uri={}&state={}",
            self.base_url.trim_end_matches('/'),
            urlencoding::encode(&challenge),
            urlencoding::encode(redirect_uri),
            urlencoding::encode(&state_id),
        );

        if let Err(e) = events::emit_login_started(&self.kernel, &state_id, redirect_uri).await {
            tracing::warn!("account: emit login_started failed: {e}");
        }

        Ok(StartLoginResponse { state_id, browser_url })
    }

    /// Exchange the auth code for an API key. On success the token is stored, the
    /// owner is bound, and the user identity is returned for display.
    pub async fn complete_login(&self, state_id: &str, code: &str) -> Result<MeView, AccountError> {
        let pending = self
            .pending
            .lock()
            .await
            .remove(state_id)
            .ok_or(AccountError::InvalidState)?;
        if pending.created_at.elapsed().as_secs() > PENDING_TTL_SECS {
            return Err(AccountError::InvalidState);
        }

        let resp = self
            .client
            .token_exchange(TokenExchangeRequest {
                code: code.into(),
                verifier: pending.code_verifier,
                ..Default::default()
            })
            .await
            .map_err(|e| AccountError::Gateway(format!("token_exchange failed: {e}")))?;
        let view = resp.view();
        let token = AccountToken {
            key: view.key.to_string(),
            user_id: view.user_id.to_string(),
            expires_at: view.expires_at,
            stored_at: chrono::Utc::now().timestamp(),
        };
        self.store.write(token.clone()).await?;

        // Bind this daemon to the logged-in user (idempotent single-row owner
        // projection). Injected hook keeps the crate decoupled.
        self.owner.bind(&token.user_id).await;

        if let Err(e) = events::emit_login_completed(&self.kernel, &token.user_id).await {
            tracing::warn!("account: emit login_completed failed: {e}");
        }

        // Best-effort identity fetch; the token is stored regardless.
        match self.refresh_me().await {
            Ok(me) => Ok(me),
            Err(_) => Ok(MeView {
                id: token.user_id,
                name: String::new(),
                email: String::new(),
                image: None,
            }),
        }
    }

    /// Fetch the current user identity from the gateway, authenticated with the
    /// stored key. Errors when logged out / expired / gateway-rejected.
    pub async fn refresh_me(&self) -> Result<MeView, AccountError> {
        let stored = self.store.read().await?.ok_or(AccountError::LoggedOut)?;
        if stored.is_expired() {
            return Err(AccountError::Expired);
        }
        let opts =
            CallOptions::default().with_header("authorization", format!("Bearer {}", stored.key));
        let resp = self
            .client
            .get_me_with_options(GetMeRequest::default(), opts)
            .await
            .map_err(|e| AccountError::Gateway(format!("GetMe failed: {e}")))?;
        let view = resp.view();
        let user = view
            .user
            .as_option()
            .ok_or_else(|| AccountError::Gateway("GetMe: empty user".into()))?;
        Ok(MeView {
            id: user.id.to_string(),
            name: user.name.to_string(),
            email: user.email.to_string(),
            image: if user.image.is_empty() { None } else { Some(user.image.to_string()) },
        })
    }

    /// Snapshot of account state. Expired tokens read as logged-out (cleanup
    /// happens on the next logout).
    pub async fn state(&self) -> Result<AccountState, AccountError> {
        match self.store.read().await? {
            None => Ok(AccountState::LoggedOut),
            Some(t) if t.is_expired() => Ok(AccountState::LoggedOut),
            Some(t) => Ok(AccountState::LoggedIn { user_id: t.user_id, me: None }),
        }
    }

    /// Clear the stored token. Idempotent. When a key is present, asks the
    /// gateway to revoke it BEFORE clearing the local file so a leaked key can't
    /// be reused for the rest of its TTL. If revoke fails (offline / gateway
    /// error) the local file is still cleared — logout must work offline — and
    /// `account.logout` records `key_revoked=false` so audit can distinguish
    /// "logged out + revoked" from "logged out (local only)". Returns the
    /// `(had_token, key_revoked)` outcome.
    pub async fn logout(&self) -> Result<LogoutOutcome, AccountError> {
        let prior = self.store.read().await.ok().flatten();
        let had_token = prior.is_some();
        let key_revoked = match prior.as_ref() {
            Some(token) => self.try_revoke_key(&token.key).await,
            None => false,
        };
        let prior_user = prior.map(|t| t.user_id);
        self.store.clear().await?;
        if let Some(ref user) = prior_user {
            self.owner.unbind(user).await;
        }
        if let Err(e) =
            events::emit_logout(&self.kernel, prior_user.as_deref(), had_token, key_revoked).await
        {
            tracing::warn!("account: emit logout failed: {e}");
        }
        Ok(LogoutOutcome { had_token, key_revoked })
    }

    /// Ask the gateway to revoke `key`. Returns true on success; logs + returns
    /// false on any failure so logout can still clear the local file.
    async fn try_revoke_key(&self, key: &str) -> bool {
        let opts = CallOptions::default().with_header("authorization", format!("Bearer {key}"));
        match self.client.revoke_key_with_options(RevokeKeyRequest::default(), opts).await {
            Ok(resp) => resp.view().revoked,
            Err(e) => {
                tracing::warn!(
                    "account: gateway revoke_key failed ({}); clearing local token only: {e}",
                    self.base_url
                );
                false
            }
        }
    }

    /// Return the stored API key — the SINGLE key-read path the publish flow
    /// depends on (single-own; no `credentials.jsonc` mirror). Errors when
    /// logged out or expired.
    pub async fn get_key(&self) -> Result<String, AccountError> {
        let token = self.store.read().await?.ok_or(AccountError::LoggedOut)?;
        if token.is_expired() {
            return Err(AccountError::Expired);
        }
        Ok(token.key)
    }

    async fn gc_pending(&self) {
        let mut p = self.pending.lock().await;
        p.retain(|_, pending| pending.created_at.elapsed().as_secs() <= PENDING_TTL_SECS);
    }

    /// Number of in-flight pending logins. Diagnostics / test introspection.
    pub async fn pending_len(&self) -> usize {
        self.pending.lock().await.len()
    }
}

/// The observable result of a `logout` call. `LogoutResult` on the wire.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LogoutOutcome {
    pub had_token: bool,
    pub key_revoked: bool,
}

fn generate_code_verifier() -> String {
    let mut buf = [0u8; 48];
    getrandom::getrandom(&mut buf).expect("OS RNG failed");
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(buf)
}

fn s256_challenge(verifier: &str) -> String {
    let hash = Sha256::digest(verifier.as_bytes());
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(hash)
}

fn generate_state_id() -> String {
    let mut buf = [0u8; 16];
    getrandom::getrandom(&mut buf).expect("OS RNG failed");
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(buf)
}
