//! AccountService — orchestrates login / me / logout flows for the daemon.
//!
//! The CLI calls daemon HTTP routes that delegate here. The service owns:
//!   - The token store (file-backed today; keychain later)
//!   - The pending-login registry (in-memory; ephemeral)
//!   - The Connect-RPC client to the gateway
//!
//! PKCE flow split (see `account/mod.rs` for the rationale):
//!   1. `start_login` — CLI passes the redirect_uri (its own callback port);
//!      daemon stores PKCE state, returns the browser auth URL
//!   2. user completes OAuth in browser → CLI's callback server gets code
//!   3. `complete_login` — CLI passes the auth code; daemon exchanges
//!      with the gateway using stored verifier, stores resulting API key
//!
//! Why the CLI keeps the callback server: the daemon's port is per-worktree
//! and not stable across restarts; OAuth providers need pre-registered
//! redirect_uris. CLI uses an ephemeral port the redirect_uri parameter
//! threads through. Future per-user-TLS would let the daemon own the
//! callback at a stable URL; filed as a follow-up.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

use anyhow::{Context, Result, anyhow};
use base64::Engine as _;
use sha2::{Digest, Sha256};
use tokio::sync::Mutex;

use ship_gateway_client::{
    AuthServiceClient, CallOptions, GetMeRequest, HttpClient, RevokeKeyRequest,
    TokenExchangeRequest, auth_client,
};

use super::events as account_events;
use super::store::AccountTokenStore;
use super::types::{AccountState, AccountToken, MeView};

/// In-memory record of a login that's been started but not yet exchanged.
/// Lives only on the daemon and is dropped when the daemon restarts; if
/// the CLI's browser flow is interrupted across a daemon restart, the
/// user just retries `ship login`.
#[derive(Debug, Clone)]
struct Pending {
    code_verifier: String,
    created_at: Instant,
}

/// Maximum age for a pending login before it's garbage-collected. Five
/// minutes covers the realistic OAuth-in-browser duration; old entries
/// would otherwise leak memory across long-running daemons.
const PENDING_TTL_SECS: u64 = 5 * 60;

#[derive(Debug, Clone, serde::Serialize)]
pub struct StartLoginResponse {
    /// Opaque id the client passes back in `complete_login`.
    pub state_id: String,
    /// URL the CLI should open in the user's browser. Includes the
    /// PKCE challenge and the CLI-supplied redirect_uri.
    pub browser_url: String,
}

pub struct AccountService {
    store: Arc<dyn AccountTokenStore>,
    pending: Arc<Mutex<HashMap<String, Pending>>>,
    /// Connect-RPC client to the gateway. Built once per service; reused
    /// across calls (HttpClient pools connections internally).
    client: AuthServiceClient<HttpClient>,
    /// Base URL the client points at. Logged on errors and surfaced for
    /// diagnostics; not used to dynamically retarget.
    base_url: String,
}

impl AccountService {
    /// Build a service that talks to the gateway at `base_url`. Production
    /// callers pass `https://api.getship.dev`; staging passes
    /// `https://staging.getship.dev`; dev passes `http://localhost:8787`.
    pub fn new(store: Arc<dyn AccountTokenStore>, base_url: impl Into<String>) -> Result<Self> {
        let base_url = base_url.into();
        let client = auth_client(&base_url).context("building gateway client")?;
        Ok(Self {
            store,
            pending: Arc::new(Mutex::new(HashMap::new())),
            client,
            base_url,
        })
    }

    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    /// Start a login flow. Generates a PKCE verifier/challenge pair,
    /// registers a state id, and returns the URL the CLI should open in
    /// the user's browser.
    ///
    /// `redirect_uri` must be the CLI's local callback URL (e.g.
    /// `http://127.0.0.1:54321/callback`). The gateway will validate
    /// this is `localhost`-shaped and reject anything else.
    pub async fn start_login(&self, redirect_uri: &str) -> Result<StartLoginResponse> {
        let verifier = generate_code_verifier();
        let challenge = s256_challenge(&verifier);
        let state_id = generate_state_id();

        self.gc_pending().await;
        self.pending.lock().await.insert(
            state_id.clone(),
            Pending {
                code_verifier: verifier,
                created_at: Instant::now(),
            },
        );

        let browser_url = format!(
            "{}/v1/auth/cli?code_challenge={}&code_challenge_method=S256&redirect_uri={}&state={}",
            self.base_url.trim_end_matches('/'),
            urlencoding::encode(&challenge),
            urlencoding::encode(redirect_uri),
            urlencoding::encode(&state_id),
        );

        account_events::emit_login_started(&state_id, redirect_uri);

        Ok(StartLoginResponse {
            state_id,
            browser_url,
        })
    }

    /// Exchange the auth code (returned by the browser callback) for an
    /// API key. On success, the token is stored and the user identity is
    /// returned for display + daemon binding.
    pub async fn complete_login(&self, state_id: &str, code: &str) -> Result<MeView> {
        let pending = self
            .pending
            .lock()
            .await
            .remove(state_id)
            .ok_or_else(|| anyhow!("invalid or expired login state"))?;
        if pending.created_at.elapsed().as_secs() > PENDING_TTL_SECS {
            return Err(anyhow!("login state expired"));
        }

        let resp = self
            .client
            .token_exchange(TokenExchangeRequest {
                code: code.into(),
                verifier: pending.code_verifier,
                ..Default::default()
            })
            .await
            .map_err(|e| anyhow!("gateway token_exchange failed: {e}"))?;
        // OwnedView derefs to the view type; fields are &str borrows of
        // the response body's backing bytes. `view()` returns &OwnedView
        // which derefs to the View.
        let view = resp.view();
        let token = AccountToken {
            key: view.key.to_string(),
            user_id: view.user_id.to_string(),
            expires_at: view.expires_at,
            stored_at: chrono::Utc::now().timestamp(),
        };
        self.store.write(token.clone()).await?;

        // Bind this daemon to the logged-in user. Idempotent — the second
        // call with the same id is a no-op event-store-wise. The CLI
        // previously did this itself by calling /v1/me + set_daemon_owner;
        // moving it here removes a round-trip and gives `complete_login`
        // a single coherent atomic step.
        if let Err(e) = crate::auth::attest::set_daemon_owner(&token.user_id) {
            tracing::warn!("account: daemon-owner bind failed: {e}");
        }

        account_events::emit_login_completed(&token.user_id);

        // Fetch /me to return the user identity. Best-effort — if it
        // fails, the token is still stored and the CLI can poll later.
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

    /// Fetch the current user identity from the gateway, authenticated
    /// with the stored API key. Returns an error when not logged in or
    /// when the gateway rejects the call.
    pub async fn refresh_me(&self) -> Result<MeView> {
        let stored = self
            .store
            .read()
            .await?
            .ok_or_else(|| anyhow!("not logged in"))?;
        if stored.is_expired() {
            return Err(anyhow!("token expired; run `ship login` to re-auth"));
        }

        // Per-call header injection. Generated client exposes
        // `<method>_with_options` taking CallOptions; we use
        // `with_header` to set `Authorization: Bearer <key>`. ClientConfig
        // doesn't carry default headers in connectrpc 0.4, so per-call is
        // the right knob today; if we add a second authenticated method
        // we'll wrap the client in a thin adapter that sets the header
        // for every call.
        let opts = CallOptions::default()
            .with_header("authorization", format!("Bearer {}", stored.key));
        let resp = self
            .client
            .get_me_with_options(GetMeRequest::default(), opts)
            .await
            .map_err(|e| anyhow!("gateway GetMe failed: {e}"))?;
        let view = resp.view();
        let user = view
            .user
            .as_option()
            .ok_or_else(|| anyhow!("gateway GetMe: empty user"))?;
        let me = MeView {
            id: user.id.to_string(),
            name: user.name.to_string(),
            email: user.email.to_string(),
            image: if user.image.is_empty() {
                None
            } else {
                Some(user.image.to_string())
            },
        };
        account_events::emit_key_refreshed(&me.id);
        Ok(me)
    }

    /// Snapshot of account state — used by Studio and `ship whoami`.
    pub async fn state(&self) -> Result<AccountState> {
        let stored = self.store.read().await?;
        match stored {
            None => Ok(AccountState::LoggedOut),
            Some(t) if t.is_expired() => {
                // Treat expired as logged-out to the surface; cleanup
                // happens on next logout call.
                Ok(AccountState::LoggedOut)
            }
            Some(t) => Ok(AccountState::LoggedIn {
                user_id: t.user_id,
                me: None,
            }),
        }
    }

    /// Clear the stored token. Idempotent. When a key is present, asks
    /// the gateway to revoke it BEFORE clearing the local file so a
    /// leaked-then-logged-out key can't be reused for the rest of its
    /// 1-year TTL. If the revoke call fails (network down, gateway
    /// error) we still clear the local file — logout must work offline,
    /// even if it means the server-side key lingers — and the emitted
    /// `account.logout` event records `key_revoked: false` so audit can
    /// distinguish the two outcomes. Emits `daemon.owner.unbound` only
    /// when a binding existed.
    pub async fn logout(&self) -> Result<()> {
        let prior = self.store.read().await.ok().flatten();
        let key_revoked = match prior.as_ref() {
            Some(token) => self.try_revoke_key(&token.key).await,
            None => false,
        };
        let prior_user = prior.map(|t| t.user_id);
        self.store.clear().await?;
        if let Err(e) = crate::auth::attest::clear_daemon_owner() {
            tracing::warn!("account: daemon-owner unbind failed: {e}");
        }
        account_events::emit_logout(prior_user.as_deref(), prior_user.is_some(), key_revoked);
        Ok(())
    }

    /// Ask the gateway to revoke `key`. Returns true on success; logs and
    /// returns false on any failure so logout can still clear the local
    /// file. Pulled out of `logout` to keep the lifecycle method readable.
    async fn try_revoke_key(&self, key: &str) -> bool {
        let opts = CallOptions::default().with_header("authorization", format!("Bearer {key}"));
        match self
            .client
            .revoke_key_with_options(RevokeKeyRequest::default(), opts)
            .await
        {
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

    /// Return the stored API key — transitional, used by the CLI to
    /// dual-write to its legacy `credentials.jsonc` until `ship publish`
    /// and `ship convert <getship-url>` are routed through the daemon.
    /// Returns Err when not logged in.
    ///
    /// **This is a tactical bridge.** It exposes a credential over local
    /// HTTP, which is fine today (CallerKind::Cli gate on the route +
    /// the floor's `Read(~/.ship/machine/**)` deny already covering the
    /// file the CLI re-stores into) but should be removed once all
    /// gateway call sites go via the daemon. Tracked separately.
    pub async fn get_key(&self) -> Result<String> {
        let token = self
            .store
            .read()
            .await?
            .ok_or_else(|| anyhow!("not logged in"))?;
        if token.is_expired() {
            return Err(anyhow!("token expired"));
        }
        Ok(token.key)
    }

    async fn gc_pending(&self) {
        let mut p = self.pending.lock().await;
        p.retain(|_, pending| pending.created_at.elapsed().as_secs() <= PENDING_TTL_SECS);
    }
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

#[cfg(test)]
#[path = "service_tests.rs"]
mod tests;
