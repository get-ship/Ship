//! ConnectRPC wire layer for `AccountService` (account.proto).
//!
//! `AccountWire` wraps the process-global `AccountService` and implements the
//! generated typed trait so shipd mounts it via `AccountServiceExt::register`.
//! Account identity is process-global (not per-request / per-space), so the wire
//! holds one `Arc<AccountService>` directly — no per-request context factory.
//!
//! Every RPC is **operator-cred-gated** (S6, #124): account ops are credential-
//! dangerous (login/logout touch the Ship Cloud key), so they require operator
//! authority — granted SOLELY by a verified, pid-attested operator cred. The
//! Connect router does not run a per-route auth middleware, so the gate is read
//! here from the resolved authority the daemon installs as a `RequestContext`
//! extension. This crate cannot see shipd's `AuthContext` type, so the daemon
//! projects the operator verdict into the shared `ship_types::Authority` and
//! inserts it; the wire checks `Authority::Operator`. The CLI transport and the
//! local peer-uid grant NO elevation — only the operator cred clears this gate.

use std::sync::Arc;

use connectrpc::{ConnectError, RequestContext, Response, ServiceResult};
use ship_types::Authority;

use ship_ops_rpc::{
    AccountService as AccountWireTrait, AccountState as ProtoState, LoginCompleted, LoginStarted,
    LogoutResult, Me, OwnedLoginCompleteRequestView, OwnedLoginRequestView, OwnedLogoutRequestView,
    OwnedStateRequestView,
};

use crate::service::AccountService;
use crate::types::{AccountState, MeView};

/// Wraps the process-global `AccountService` as a Connect service.
pub struct AccountWire {
    svc: Arc<AccountService>,
}

impl AccountWire {
    pub fn new(svc: Arc<AccountService>) -> Self {
        Self { svc }
    }
}

/// Reject any caller that isn't the operator. Account ops are credential-
/// dangerous, so they require operator authority — granted SOLELY by a verified
/// operator cred (S6, #124). The daemon resolves the caller's authority and
/// installs it as a `RequestContext` extension; the wire reads the shared
/// `ship_types::Authority` (no coupling to shipd's `AuthContext`). Absence ⟹ no
/// auth context (unauthenticated); any non-operator authority ⟹ denied.
fn require_operator(ctx: &RequestContext) -> Result<(), ConnectError> {
    match ctx.extensions.get::<Authority>() {
        Some(Authority::Operator) => Ok(()),
        Some(_) => Err(ConnectError::permission_denied(
            "account ops require operator authority (a verified operator cred)",
        )),
        None => Err(ConnectError::unauthenticated("auth context required")),
    }
}

fn me_to_proto(me: MeView) -> Me {
    Me {
        id: me.id,
        name: me.name,
        email: me.email,
        image: me.image.unwrap_or_default(),
        ..Default::default()
    }
}

fn state_to_proto(state: AccountState) -> ProtoState {
    use ship_ops_rpc::account_state::Status;
    match state {
        AccountState::LoggedOut => ProtoState {
            status: buffa::EnumValue::Known(Status::LOGGED_OUT),
            ..Default::default()
        },
        AccountState::Pending { state_id } => ProtoState {
            status: buffa::EnumValue::Known(Status::PENDING),
            state_id,
            ..Default::default()
        },
        AccountState::LoggedIn { user_id, me } => ProtoState {
            status: buffa::EnumValue::Known(Status::LOGGED_IN),
            user_id,
            me: me.map(me_to_proto).into(),
            ..Default::default()
        },
    }
}

#[allow(refining_impl_trait)]
impl AccountWireTrait for AccountWire {
    async fn state(
        &self,
        ctx: RequestContext,
        _request: OwnedStateRequestView,
    ) -> ServiceResult<ProtoState> {
        require_operator(&ctx)?;
        let state = self.svc.state().await.map_err(ConnectError::from)?;
        Ok(Response::new(state_to_proto(state)))
    }

    async fn login(
        &self,
        ctx: RequestContext,
        request: OwnedLoginRequestView,
    ) -> ServiceResult<LoginStarted> {
        require_operator(&ctx)?;
        let redirect_uri = request.redirect_uri;
        if !redirect_uri.starts_with("http://127.0.0.1:")
            && !redirect_uri.starts_with("http://localhost:")
        {
            return Err(ConnectError::invalid_argument(
                "redirect_uri must be a localhost http URL",
            ));
        }
        let resp = self.svc.start_login(redirect_uri).await.map_err(ConnectError::from)?;
        Ok(Response::new(LoginStarted {
            state_id: resp.state_id,
            browser_url: resp.browser_url,
            ..Default::default()
        }))
    }

    async fn login_complete(
        &self,
        ctx: RequestContext,
        request: OwnedLoginCompleteRequestView,
    ) -> ServiceResult<LoginCompleted> {
        require_operator(&ctx)?;
        let me = self
            .svc
            .complete_login(request.state_id, request.code)
            .await
            .map_err(ConnectError::from)?;
        let user_id = me.id.clone();
        Ok(Response::new(LoginCompleted {
            user_id,
            me: Some(me_to_proto(me)).into(),
            ..Default::default()
        }))
    }

    async fn logout(
        &self,
        ctx: RequestContext,
        _request: OwnedLogoutRequestView,
    ) -> ServiceResult<LogoutResult> {
        require_operator(&ctx)?;
        let outcome = self.svc.logout().await.map_err(ConnectError::from)?;
        Ok(Response::new(LogoutResult {
            had_token: outcome.had_token,
            key_revoked: outcome.key_revoked,
            ..Default::default()
        }))
    }
}
