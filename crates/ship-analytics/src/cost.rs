//! Cost computation — model-rate table + token-to-USD conversion.
//!
//! Stub today. The analytics_sessions table doesn't carry token
//! counts yet (the harness streams them via stream-json `usage`
//! blocks; the supervisor's reader middleware drops that data
//! today). When stream-side capture lands as `harness.usage_observed`
//! kernel events + new columns on analytics_sessions, this module
//! starts doing real math.
//!
//! Why scaffold now: the model-rate table is the source-of-truth
//! that has to live somewhere — model names, per-token prices,
//! pricing-tier rules. Putting the file here pins the home so the
//! day stream capture lands, the cost layer doesn't need a fresh
//! design pass.
//!
//! Pricing source: Anthropic / OpenAI / Google publish current rates.
//! Update via PR; cache invalidation is "redeploy."

use serde::{Deserialize, Serialize};

/// USD-per-million-tokens for a model. Prices change; update when
/// providers announce changes.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ModelRate {
    pub input_per_mtok: f64,
    pub output_per_mtok: f64,
    /// Cache-creation token price. `None` when the model doesn't
    /// support prompt caching.
    pub cache_creation_per_mtok: Option<f64>,
    /// Cache-read token price. `None` when not applicable.
    pub cache_read_per_mtok: Option<f64>,
}

/// Look up the per-million-tokens rate for a model name. Names are
/// matched case-insensitively against a hand-maintained table. `None`
/// for unknown models — analytics should still show counts, just
/// without cost.
///
/// Stub: only a small table for the obvious models we exercise in
/// dev. Real rollout would source this from a versioned data file
/// (e.g. crates/ship-analytics/data/model-rates.toml) so updates
/// don't need a code change.
pub fn lookup_rate(model: &str) -> Option<ModelRate> {
    let key = model.to_lowercase();
    match key.as_str() {
        // Anthropic — illustrative; verify against current pricing.
        "claude-opus-4.7" | "claude-opus-4-7" | "opus" => Some(ModelRate {
            input_per_mtok: 15.0,
            output_per_mtok: 75.0,
            cache_creation_per_mtok: Some(18.75),
            cache_read_per_mtok: Some(1.50),
        }),
        "claude-sonnet-4.6" | "claude-sonnet-4-6" | "sonnet" => Some(ModelRate {
            input_per_mtok: 3.0,
            output_per_mtok: 15.0,
            cache_creation_per_mtok: Some(3.75),
            cache_read_per_mtok: Some(0.30),
        }),
        "claude-haiku-4.5" | "claude-haiku-4-5" | "haiku" => Some(ModelRate {
            input_per_mtok: 1.0,
            output_per_mtok: 5.0,
            cache_creation_per_mtok: Some(1.25),
            cache_read_per_mtok: Some(0.10),
        }),
        _ => None,
    }
}

/// Compute total USD cost for a session given its token usage and model.
/// Returns `None` when the model is unknown — caller decides whether
/// to render "—" or fall back to a default rate.
pub fn session_cost_usd(
    model: &str,
    input_tokens: u64,
    output_tokens: u64,
    cache_creation_tokens: u64,
    cache_read_tokens: u64,
) -> Option<f64> {
    let rate = lookup_rate(model)?;
    let mil = 1_000_000.0;
    let mut total = (input_tokens as f64 / mil) * rate.input_per_mtok
        + (output_tokens as f64 / mil) * rate.output_per_mtok;
    if let Some(rc) = rate.cache_creation_per_mtok {
        total += (cache_creation_tokens as f64 / mil) * rc;
    }
    if let Some(rr) = rate.cache_read_per_mtok {
        total += (cache_read_tokens as f64 / mil) * rr;
    }
    Some(total)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_model_yields_some_rate() {
        assert!(lookup_rate("claude-opus-4-7").is_some());
        assert!(lookup_rate("Claude-Opus-4-7").is_some()); // case-insensitive
        assert!(lookup_rate("nonexistent-model").is_none());
    }

    #[test]
    fn session_cost_matches_per_token_math() {
        // 1M input × $15 = $15
        let c = session_cost_usd("opus", 1_000_000, 0, 0, 0).unwrap();
        assert!((c - 15.0).abs() < 0.001);
        // 1M output × $75 = $75
        let c = session_cost_usd("opus", 0, 1_000_000, 0, 0).unwrap();
        assert!((c - 75.0).abs() < 0.001);
    }

    #[test]
    fn unknown_model_returns_none() {
        assert!(session_cost_usd("vapor-model-7000", 100, 100, 0, 0).is_none());
    }
}
