//! `AccountError` — the crate's error type, mapped to `ConnectError` in the
//! wire handler.

use connectrpc::ConnectError;

#[derive(Debug, thiserror::Error)]
pub enum AccountError {
    /// Not logged in (no token, or expired token treated as logged-out for the
    /// key-read path).
    #[error("not logged in")]
    LoggedOut,
    /// The stored token is past its expiry.
    #[error("token expired; run `ship login` to re-auth")]
    Expired,
    /// The login state id is unknown or its TTL elapsed.
    #[error("invalid or expired login state")]
    InvalidState,
    /// Caller-supplied input failed validation (e.g. non-localhost redirect_uri).
    #[error("validation: {0}")]
    Validation(String),
    /// A gateway call (token_exchange / get_me / revoke) failed.
    #[error("gateway: {0}")]
    Gateway(String),
    /// Token-store or event-emit I/O failure.
    #[error("internal: {0}")]
    Internal(String),
}

impl From<AccountError> for ConnectError {
    fn from(e: AccountError) -> Self {
        match e {
            AccountError::LoggedOut => ConnectError::unauthenticated(e.to_string()),
            AccountError::Expired => ConnectError::unauthenticated(e.to_string()),
            AccountError::InvalidState => ConnectError::invalid_argument(e.to_string()),
            AccountError::Validation(m) => ConnectError::invalid_argument(m),
            AccountError::Gateway(m) => ConnectError::internal(m),
            AccountError::Internal(m) => ConnectError::internal(m),
        }
    }
}

impl From<anyhow::Error> for AccountError {
    fn from(e: anyhow::Error) -> Self {
        AccountError::Internal(e.to_string())
    }
}

impl From<ship_substrate::errors::KernelError> for AccountError {
    fn from(e: ship_substrate::errors::KernelError) -> Self {
        AccountError::Internal(e.to_string())
    }
}
