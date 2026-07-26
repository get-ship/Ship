//! Minimal Yoke (floki) provider smoke test — one completion, no Ship, no tools.
//!
//! Usage:
//! ```text
//! export YOKE_PROVIDER=nvidia
//! export NVIDIA_API_KEY=nvapi-...
//! export YOKE_MODEL=meta/llama-3.1-8b-instruct   # catalog id you have access to
//! cargo run -p floki --bin yoke-smoke
//! ```
//!
//! Optional prompt: `cargo run -p floki --bin yoke-smoke -- "your prompt"`

use anyhow::{Context, Result};
use floki::provider::{BuiltModel, Byok, CredentialSource, Provider, build_model};
use rig::agent::AgentBuilder;
use rig::completion::Prompt;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    let cred = Byok::from_env()
        .resolve()
        .context(
            "credential resolution failed — set YOKE_PROVIDER, YOKE_MODEL, \
             and the provider-named key (e.g. NVIDIA_API_KEY for nvidia)",
        )?;

    eprintln!(
        "yoke-smoke: provider={} model={}",
        cred.provider.id(),
        cred.model
    );
    match &cred.provider {
        Provider::Nvidia { base_url } | Provider::OpenAiCompat { base_url } => {
            eprintln!("yoke-smoke: base_url={base_url}");
        }
        Provider::OpenAi {
            base_url: Some(u),
        } => {
            eprintln!("yoke-smoke: base_url={u}");
        }
        Provider::OpenAi { base_url: None } | Provider::Anthropic => {}
    }

    let prompt = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "Reply with exactly one word: pong".to_string());

    let reply = match build_model(&cred).context("build_model")? {
        BuiltModel::Anthropic(m) => prompt_once(m, &prompt).await?,
        BuiltModel::OpenAiCompat(m) => prompt_once(m, &prompt).await?,
    };

    println!("{reply}");
    Ok(())
}

async fn prompt_once<M>(model: M, user: &str) -> Result<String>
where
    M: rig::completion::CompletionModel + 'static,
{
    let agent = AgentBuilder::new(model)
        .preamble("You are a harness smoke test. Obey the user briefly.")
        .build();
    agent
        .prompt(user)
        .await
        .map_err(|e| anyhow::anyhow!("completion failed: {e}"))
}
