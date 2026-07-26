//! Provider / BYOK credential resolution — S7.
//!
//! Resolves which LLM provider to use and what API key to present, via an
//! injected key/value getter (never global env directly — callers that want
//! env can use [`Byok::from_env`]). Model construction is gated here so the
//! rest of the crate stays decoupled from rig provider types.
//!
//! # Env contract (Yoke / Bridge — not Ship)
//!
//! **Selection** (tool-scoped prefix `YOKE_`):
//! - `YOKE_PROVIDER` — `anthropic` | `openai` | `nvidia` | `openai-compat`
//! - `YOKE_MODEL` — model id for the active provider (required)
//!
//! **Secrets / endpoints are provider-named** (never put an NVIDIA key in
//! `OPENAI_API_KEY`):
//! - `anthropic` → `ANTHROPIC_API_KEY`
//! - `openai` → `OPENAI_API_KEY`, optional `OPENAI_BASE_URL`
//! - `nvidia` → `NVIDIA_API_KEY`, optional `NVIDIA_BASE_URL`
//!   (default `https://integrate.api.nvidia.com/v1`)
//! - `openai-compat` (generic OpenAI-wire host) → `OPENAI_COMPAT_API_KEY`,
//!   `OPENAI_COMPAT_BASE_URL` (required)
//!
//! Deprecated Ship-era aliases (`SHIP_PROVIDER`, `SHIP_MODEL`, `SHIP_BASE_URL`)
//! still resolve with a warning so shipd keeps working until it injects its
//! own [`CredentialSource`]. New code must use `YOKE_*` + provider-named keys.

use anyhow::{Context, bail};
use rig::client::CompletionClient;
use rig::providers::{anthropic, openai};

// ---------------------------------------------------------------------------
// Core types
// ---------------------------------------------------------------------------

/// Default NVIDIA Integrate / Build OpenAI-compatible root.
pub const NVIDIA_DEFAULT_BASE_URL: &str = "https://integrate.api.nvidia.com/v1";

/// Which LLM provider backs this credential.
///
/// Wire protocol is an implementation detail: Anthropic uses its native
/// client; everything else uses OpenAI-compatible chat completions with a
/// base URL.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Provider {
    Anthropic,
    /// Named OpenAI (api.openai.com or custom via base_url).
    OpenAi { base_url: Option<String> },
    /// NVIDIA Build / NIM / Integrate — OpenAI wire, NVIDIA secrets.
    Nvidia { base_url: String },
    /// Escape hatch: any OpenAI-compatible host with its own key env.
    OpenAiCompat { base_url: String },
}

impl Provider {
    /// Canonical id used in env / config (`anthropic`, `nvidia`, …).
    pub fn id(&self) -> &'static str {
        match self {
            Provider::Anthropic => "anthropic",
            Provider::OpenAi { .. } => "openai",
            Provider::Nvidia { .. } => "nvidia",
            Provider::OpenAiCompat { .. } => "openai-compat",
        }
    }
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

    fn get(&self, key: &str) -> Option<String> {
        (self.getter)(key)
    }

    /// Prefer `primary`; if missing, try `legacy` and note deprecation.
    fn get_prefer(&self, primary: &str, legacy: &str) -> Option<String> {
        if let Some(v) = self.get(primary) {
            return Some(v);
        }
        if let Some(v) = self.get(legacy) {
            tracing::warn!(
                primary,
                legacy,
                "deprecated env `{legacy}` used; prefer `{primary}` (Yoke/Bridge, not Ship)"
            );
            return Some(v);
        }
        None
    }
}

impl<F> CredentialSource for Byok<F>
where
    F: Fn(&str) -> Option<String>,
{
    fn resolve(&self) -> anyhow::Result<Credential> {
        let provider_str = self
            .get_prefer("YOKE_PROVIDER", "SHIP_PROVIDER")
            .unwrap_or_else(|| "anthropic".into());
        let model = self
            .get_prefer("YOKE_MODEL", "SHIP_MODEL")
            .context("YOKE_MODEL is required (model id for the active provider)")?;

        match provider_str.as_str() {
            "anthropic" => {
                let api_key = self
                    .get("ANTHROPIC_API_KEY")
                    .context("ANTHROPIC_API_KEY is required for provider `anthropic`")?;
                Ok(Credential {
                    provider: Provider::Anthropic,
                    api_key,
                    model,
                })
            }
            "openai" => {
                let api_key = self
                    .get("OPENAI_API_KEY")
                    .context("OPENAI_API_KEY is required for provider `openai`")?;
                let base_url = self.get("OPENAI_BASE_URL");
                Ok(Credential {
                    provider: Provider::OpenAi { base_url },
                    api_key,
                    model,
                })
            }
            "nvidia" => {
                let api_key = self
                    .get("NVIDIA_API_KEY")
                    .or_else(|| self.get("NGC_API_KEY"))
                    .context(
                        "NVIDIA_API_KEY is required for provider `nvidia` \
                         (do not put NVIDIA keys in OPENAI_API_KEY)",
                    )?;
                let base_url = self
                    .get("NVIDIA_BASE_URL")
                    .unwrap_or_else(|| NVIDIA_DEFAULT_BASE_URL.to_string());
                Ok(Credential {
                    provider: Provider::Nvidia { base_url },
                    api_key,
                    model,
                })
            }
            "openai-compat" => {
                // Generic OpenAI-wire host. Prefer OPENAI_COMPAT_*; accept
                // legacy OPENAI_API_KEY + SHIP_BASE_URL only as deprecated.
                let base_url = self
                    .get_prefer("OPENAI_COMPAT_BASE_URL", "SHIP_BASE_URL")
                    .context(
                        "OPENAI_COMPAT_BASE_URL is required for provider `openai-compat`",
                    )?;
                let api_key = if let Some(k) = self.get("OPENAI_COMPAT_API_KEY") {
                    k
                } else if let Some(k) = self.get("OPENAI_API_KEY") {
                    tracing::warn!(
                        "provider `openai-compat` using OPENAI_API_KEY; \
                         prefer OPENAI_COMPAT_API_KEY so real OpenAI keys stay distinct"
                    );
                    k
                } else {
                    bail!(
                        "OPENAI_COMPAT_API_KEY is required for provider `openai-compat`"
                    );
                };
                Ok(Credential {
                    provider: Provider::OpenAiCompat { base_url },
                    api_key,
                    model,
                })
            }
            other => bail!(
                "unknown YOKE_PROVIDER value: {other:?}; expected \
                 `anthropic`, `openai`, `nvidia`, or `openai-compat`"
            ),
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
// Model construction
// ---------------------------------------------------------------------------

/// A rig completion model built from a `Credential`.
pub enum BuiltModel {
    Anthropic(anthropic::completion::CompletionModel),
    OpenAiCompat(openai::completion::CompletionModel),
}

/// Build a rig provider model from `cred`. No live API call is made.
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
        Provider::OpenAi { base_url } => {
            let mut builder = openai::CompletionsClient::builder().api_key(cred.api_key.clone());
            if let Some(url) = base_url {
                builder = builder.base_url(url);
            }
            let client = builder
                .build()
                .context("failed to build OpenAI client")?;
            let model = client.completion_model(&cred.model);
            Ok(BuiltModel::OpenAiCompat(model))
        }
        Provider::Nvidia { base_url } | Provider::OpenAiCompat { base_url } => {
            let client = openai::CompletionsClient::builder()
                .api_key(cred.api_key.clone())
                .base_url(base_url)
                .build()
                .context("failed to build OpenAI-compatible client")?;
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
    fn anthropic_resolves_with_yoke_prefix() {
        let env = HashMap::from([
            ("YOKE_PROVIDER", "anthropic"),
            ("ANTHROPIC_API_KEY", "sk-ant-test"),
            ("YOKE_MODEL", "claude-sonnet-4-6"),
        ]);
        let cred = Byok::new(getter(env)).resolve().unwrap();
        assert!(matches!(cred.provider, Provider::Anthropic));
        assert_eq!(cred.api_key, "sk-ant-test");
        assert_eq!(cred.model, "claude-sonnet-4-6");
    }

    #[test]
    fn nvidia_uses_nvidia_key_not_openai() {
        let env = HashMap::from([
            ("YOKE_PROVIDER", "nvidia"),
            ("NVIDIA_API_KEY", "nvapi-test"),
            ("YOKE_MODEL", "meta/llama-3.1-8b-instruct"),
        ]);
        let cred = Byok::new(getter(env)).resolve().unwrap();
        match &cred.provider {
            Provider::Nvidia { base_url } => {
                assert_eq!(base_url, NVIDIA_DEFAULT_BASE_URL);
            }
            other => panic!("expected Nvidia, got {other:?}"),
        }
        assert_eq!(cred.api_key, "nvapi-test");
        assert_eq!(cred.model, "meta/llama-3.1-8b-instruct");
    }

    #[test]
    fn nvidia_rejects_openai_key_only() {
        let env = HashMap::from([
            ("YOKE_PROVIDER", "nvidia"),
            ("OPENAI_API_KEY", "sk-should-not-count"),
            ("YOKE_MODEL", "meta/llama-3.1-8b-instruct"),
        ]);
        let err = Byok::new(getter(env)).resolve().unwrap_err().to_string();
        assert!(err.contains("NVIDIA_API_KEY"), "err={err}");
        assert!(err.contains("OPENAI_API_KEY"), "err={err}");
    }

    #[test]
    fn nvidia_custom_base_url() {
        let env = HashMap::from([
            ("YOKE_PROVIDER", "nvidia"),
            ("NVIDIA_API_KEY", "nvapi-test"),
            ("NVIDIA_BASE_URL", "https://example.nvidia.invalid/v1"),
            ("YOKE_MODEL", "foo"),
        ]);
        let cred = Byok::new(getter(env)).resolve().unwrap();
        match cred.provider {
            Provider::Nvidia { base_url } => {
                assert_eq!(base_url, "https://example.nvidia.invalid/v1");
            }
            other => panic!("expected Nvidia, got {other:?}"),
        }
    }

    #[test]
    fn openai_named_provider() {
        let env = HashMap::from([
            ("YOKE_PROVIDER", "openai"),
            ("OPENAI_API_KEY", "sk-oa-test"),
            ("YOKE_MODEL", "gpt-4o-mini"),
        ]);
        let cred = Byok::new(getter(env)).resolve().unwrap();
        assert!(matches!(
            cred.provider,
            Provider::OpenAi { base_url: None }
        ));
    }

    #[test]
    fn openai_compat_generic() {
        let env = HashMap::from([
            ("YOKE_PROVIDER", "openai-compat"),
            ("OPENAI_COMPAT_API_KEY", "sk-compat"),
            ("OPENAI_COMPAT_BASE_URL", "https://openrouter.ai/api/v1"),
            ("YOKE_MODEL", "meta-llama/llama-3.1-8b"),
        ]);
        let cred = Byok::new(getter(env)).resolve().unwrap();
        assert!(matches!(
            &cred.provider,
            Provider::OpenAiCompat { base_url } if base_url == "https://openrouter.ai/api/v1"
        ));
        assert_eq!(cred.api_key, "sk-compat");
    }

    #[test]
    fn legacy_ship_prefix_still_resolves() {
        let env = HashMap::from([
            ("SHIP_PROVIDER", "anthropic"),
            ("ANTHROPIC_API_KEY", "sk-ant-legacy"),
            ("SHIP_MODEL", "claude-haiku-4-5"),
        ]);
        let cred = Byok::new(getter(env)).resolve().unwrap();
        assert!(matches!(cred.provider, Provider::Anthropic));
        assert_eq!(cred.model, "claude-haiku-4-5");
    }

    #[test]
    fn missing_api_key_is_error_not_panic() {
        let env = HashMap::from([
            ("YOKE_PROVIDER", "anthropic"),
            ("YOKE_MODEL", "claude-sonnet-4-6"),
        ]);
        let result = Byok::new(getter(env)).resolve();
        assert!(result.is_err());
        let msg = result.unwrap_err().to_string();
        assert!(msg.contains("ANTHROPIC_API_KEY"));
    }

    #[test]
    fn default_provider_is_anthropic_when_unset() {
        let env = HashMap::from([
            ("ANTHROPIC_API_KEY", "sk-ant-default"),
            ("YOKE_MODEL", "claude-haiku-4-5"),
        ]);
        let cred = Byok::new(getter(env)).resolve().unwrap();
        assert!(matches!(cred.provider, Provider::Anthropic));
    }

    #[test]
    fn tests_do_not_read_global_env() {
        let result = Byok::new(|_| None).resolve();
        assert!(result.is_err());
    }

    #[test]
    fn unknown_provider_is_error() {
        let env = HashMap::from([("YOKE_PROVIDER", "grok"), ("YOKE_MODEL", "grok-1")]);
        let result = Byok::new(getter(env)).resolve();
        assert!(result.is_err());
    }

    #[test]
    fn build_model_nvidia_does_not_call_network() {
        let cred = Credential {
            provider: Provider::Nvidia {
                base_url: NVIDIA_DEFAULT_BASE_URL.into(),
            },
            api_key: "nvapi-x".into(),
            model: "meta/llama-3.1-8b-instruct".into(),
        };
        let built = build_model(&cred).expect("construct");
        assert!(matches!(built, BuiltModel::OpenAiCompat(_)));
    }
}
