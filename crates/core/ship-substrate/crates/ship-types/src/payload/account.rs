// -- Account lifecycle ------------------------------------------------------

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct AccountLoginStarted {
    pub redirect_uri: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AccountLoginCompleted {}

#[derive(Debug, Serialize, Deserialize)]
pub struct AccountLogout {
    pub had_token: bool,
    #[serde(default)]
    pub key_revoked: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AccountKeyRefreshed {}
