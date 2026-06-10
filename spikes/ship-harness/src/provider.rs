//! Provider story — BYOK at launch, behind a seam.
//!
//! The agent must not know where its model credential comes from. At launch the
//! only `CredentialSource` is `Byok` (user-supplied key). Post-launch, a
//! `ShipHostedCredentials` impl (pooled key + metering + markup) drops in behind
//! the SAME trait — the agent, tools, and loop are untouched.

pub enum Provider {
    Anthropic,
    /// The de-facto OpenAI-compatible surface (Ollama, vLLM, OpenRouter, ...).
    OpenAiCompat { base_url: String },
}

pub struct Credential {
    pub provider: Provider,
    pub api_key: String,
    pub model: String,
}

/// Where the model credential comes from. Launch: BYOK. Later: Ship-hosted.
pub trait CredentialSource: Send + Sync {
    fn resolve(&self) -> anyhow::Result<Credential>;
}

/// Launch impl: the user brings the key (env here; encrypted at-rest in prod).
/// SHIP_PROVIDER=anthropic|openai-compat, plus the matching key/base/model env.
pub struct Byok;

impl CredentialSource for Byok {
    fn resolve(&self) -> anyhow::Result<Credential> {
        let which = std::env::var("SHIP_PROVIDER").unwrap_or_else(|_| "anthropic".into());
        match which.as_str() {
            "anthropic" => Ok(Credential {
                provider: Provider::Anthropic,
                api_key: std::env::var("ANTHROPIC_API_KEY")?,
                model: std::env::var("SHIP_MODEL")
                    .unwrap_or_else(|_| "claude-sonnet-4-6".into()),
            }),
            "openai-compat" => Ok(Credential {
                provider: Provider::OpenAiCompat {
                    base_url: std::env::var("SHIP_BASE_URL")
                        .unwrap_or_else(|_| "https://api.openai.com/v1".into()),
                },
                api_key: std::env::var("OPENAI_API_KEY").unwrap_or_default(),
                model: std::env::var("SHIP_MODEL").unwrap_or_else(|_| "gpt-4o".into()),
            }),
            other => anyhow::bail!("unknown SHIP_PROVIDER: {other}"),
        }
    }
}
