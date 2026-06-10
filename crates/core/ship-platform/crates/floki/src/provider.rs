//! Provider / BYOK credential resolution — S7.
//!
//! Resolves which LLM provider to use and what API key to present, via an
//! injected key/value getter (never global env directly — callers that want
//! env can use `Byok::from_env()`).  Model construction is also gated here so
//! the rest of the crate stays decoupled from rig provider types.

use anyhow::{Context, bail};
use rig::client::CompletionClient;
use rig::providers::{anthropic, openai};

// ---------------------------------------------------------------------------
// Core types
// ---------------------------------------------------------------------------

/// Which LLM provider backs this credential.
#[derive(Debug, Clone, PartialEq)]
pub enum Provider {
    Anthropic,
    OpenAiCompat { base_url: String },
}

/// A resolved credential: provider identity + API key + model id.
#[derive(Debug, Clone)]
pub struct Credential {
    pub provider: Provider,
    pub api_key: String,
    pub model: String,
}

/// Anything that can produce a `Credential`.
pub trait CredentialSource {
    fn resolve(&self) -> anyhow::Result<Credential>;
}

// ---------------------------------------------------------------------------
// BYOK — reads from an injected getter
// ---------------------------------------------------------------------------

/// Resolves credentials from a key/value getter (injected, never global env).
///
/// Environment variable names:
/// - `SHIP_PROVIDER`     = `anthropic` (default) | `openai-compat`
/// - `ANTHROPIC_API_KEY` / `OPENAI_API_KEY`
/// - `SHIP_BASE_URL`     (required for `openai-compat`)
/// - `SHIP_MODEL`        (required — no opinionated default to avoid silent drift)
pub struct Byok<F>
where
    F: Fn(&str) -> Option<String>,
{
    getter: F,
}

impl<F> Byok<F>
where
    F: Fn(&str) -> Option<String>,
{
    pub fn new(getter: F) -> Self {
        Self { getter }
    }
}

impl<F> CredentialSource for Byok<F>
where
    F: Fn(&str) -> Option<String>,
{
    fn resolve(&self) -> anyhow::Result<Credential> {
        let g = &self.getter;

        let provider_str = g("SHIP_PROVIDER").unwrap_or_else(|| "anthropic".into());
        let model = g("SHIP_MODEL").context("SHIP_MODEL is required")?;

        match provider_str.as_str() {
            "anthropic" => {
                let api_key = g("ANTHROPIC_API_KEY").context("ANTHROPIC_API_KEY is required")?;
                Ok(Credential {
                    provider: Provider::Anthropic,
                    api_key,
                    model,
                })
            }
            "openai-compat" => {
                let base_url = g("SHIP_BASE_URL").context("SHIP_BASE_URL is required for openai-compat")?;
                let api_key = g("OPENAI_API_KEY").context("OPENAI_API_KEY is required for openai-compat")?;
                Ok(Credential {
                    provider: Provider::OpenAiCompat { base_url },
                    api_key,
                    model,
                })
            }
            other => bail!("unknown SHIP_PROVIDER value: {other:?}; expected `anthropic` or `openai-compat`"),
        }
    }
}

impl Byok<fn(&str) -> Option<String>> {
    /// Convenience: a `Byok` backed by `std::env::var`. The only place this crate
    /// touches global env — everything else flows through the injected getter.
    pub fn from_env() -> Self {
        fn env_get(k: &str) -> Option<String> {
            std::env::var(k).ok()
        }
        Byok::new(env_get)
    }
}

// ---------------------------------------------------------------------------
// Model construction — builds the rig provider client + completion model.
// The two provider types are incommensurable so we expose a typed enum.
// Callers `match` on `Provider` to pick the arm they need.
// ---------------------------------------------------------------------------

/// A rig completion model built from a `Credential`.
/// Variants are typed concretely; the caller matches on the arm it expects.
pub enum BuiltModel {
    Anthropic(anthropic::completion::CompletionModel),
    OpenAiCompat(openai::completion::CompletionModel),
}

/// Build a rig provider model from `cred`.  No live API call is made — only the
/// client and model struct are constructed.
pub fn build_model(cred: &Credential) -> anyhow::Result<BuiltModel> {
    match &cred.provider {
        Provider::Anthropic => {
            let client = anthropic::Client::builder()
                .api_key(cred.api_key.clone())
                .build()
                .context("failed to build Anthropic client")?;
            let model = client.completion_model(&cred.model);
            Ok(BuiltModel::Anthropic(model))
        }
        Provider::OpenAiCompat { base_url } => {
            let client = openai::CompletionsClient::builder()
                .api_key(cred.api_key.clone())
                .base_url(base_url)
                .build()
                .context("failed to build OpenAI-compat client")?;
            let model = client.completion_model(&cred.model);
            Ok(BuiltModel::OpenAiCompat(model))
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn getter(map: HashMap<&'static str, &'static str>) -> impl Fn(&str) -> Option<String> {
        move |k| map.get(k).map(|v| v.to_string())
    }

    #[test]
    fn anthropic_resolves_credential() {
        let env = HashMap::from([
            ("SHIP_PROVIDER", "anthropic"),
            ("ANTHROPIC_API_KEY", "sk-ant-test"),
            ("SHIP_MODEL", "claude-sonnet-4-6"),
        ]);
        let cred = Byok::new(getter(env)).resolve().unwrap();
        assert!(matches!(cred.provider, Provider::Anthropic));
        assert_eq!(cred.api_key, "sk-ant-test");
        assert_eq!(cred.model, "claude-sonnet-4-6");
    }

    #[test]
    fn openai_compat_resolves_credential() {
        let env = HashMap::from([
            ("SHIP_PROVIDER", "openai-compat"),
            ("OPENAI_API_KEY", "sk-oa-test"),
            ("SHIP_BASE_URL", "https://openrouter.ai/api/v1"),
            ("SHIP_MODEL", "meta-llama/llama-3.1-8b"),
        ]);
        let cred = Byok::new(getter(env)).resolve().unwrap();
        assert!(matches!(
            &cred.provider,
            Provider::OpenAiCompat { base_url } if base_url == "https://openrouter.ai/api/v1"
        ));
        assert_eq!(cred.api_key, "sk-oa-test");
        assert_eq!(cred.model, "meta-llama/llama-3.1-8b");
    }

    #[test]
    fn missing_api_key_is_error_not_panic() {
        // No ANTHROPIC_API_KEY in the getter.
        let env = HashMap::from([
            ("SHIP_PROVIDER", "anthropic"),
            ("SHIP_MODEL", "claude-sonnet-4-6"),
        ]);
        let result = Byok::new(getter(env)).resolve();
        assert!(result.is_err());
        let msg = result.unwrap_err().to_string();
        assert!(msg.contains("ANTHROPIC_API_KEY"));
    }

    #[test]
    fn default_provider_is_anthropic_when_unset() {
        // SHIP_PROVIDER absent — default to anthropic.
        let env = HashMap::from([
            ("ANTHROPIC_API_KEY", "sk-ant-default"),
            ("SHIP_MODEL", "claude-haiku-4-5"),
        ]);
        let cred = Byok::new(getter(env)).resolve().unwrap();
        assert!(matches!(cred.provider, Provider::Anthropic));
    }

    #[test]
    fn tests_do_not_read_global_env() {
        // The getter here returns nothing for every key, simulating a clean env.
        // This must error (SHIP_MODEL missing), proving the injected getter — not
        // std::env — is the only source.
        let result = Byok::new(|_| None).resolve();
        assert!(result.is_err());
    }

    #[test]
    fn unknown_provider_is_error() {
        let env = HashMap::from([
            ("SHIP_PROVIDER", "grok"),
            ("SHIP_MODEL", "grok-1"),
        ]);
        let result = Byok::new(getter(env)).resolve();
        assert!(result.is_err());
    }
}
