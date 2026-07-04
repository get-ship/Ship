//! The linchpin: a `ship.ts` authored with `defineAgent`, evaluated in a
//! locked-down ship-js isolate, produces `compiler::AgentProfile`s — and a
//! program defining N agents in a loop rolls up to N profiles.

use agent_config_eval::{resolve_agents, resolve_agents_with_globals};

/// A real `ship.ts` that defines agents in a `for` loop.
const ROLLUP: &str = include_str!("fixtures/rollup.ship.ts");
/// Genuine TypeScript: `import type`, a `Partial<…>` annotation, `satisfies`,
/// typed params, `as const`. Proves the oxc strip.
const TYPED: &str = include_str!("fixtures/typed.ship.ts");
/// Spreads `globals.agentDefaults` into each `defineAgent`. Proves injection.
const GLOBALS: &str = include_str!("fixtures/globals.ship.ts");

#[tokio::test(flavor = "current_thread")]
async fn loop_rolls_up_to_one_profile_per_iteration() {
    let profiles = resolve_agents(ROLLUP).await.expect("config evaluates");

    let ids: Vec<&str> = profiles.iter().map(|p| p.agent.id.as_str()).collect();
    assert_eq!(
        ids,
        ["reviewer", "tester", "builder"],
        "loop of 3 defineAgent calls → 3 profiles, in call order"
    );
    // The collapsed fields survive the round-trip: agent.name + capabilities.
    assert_eq!(profiles[0].agent.name, "reviewer");
    assert_eq!(profiles[1].capabilities.len(), 1);
    assert_eq!(profiles[1].capabilities[0].domain, "workspace");
}

#[tokio::test(flavor = "current_thread")]
async fn eval_is_deterministic() {
    let first = resolve_agents(ROLLUP).await.expect("first eval");
    let second = resolve_agents(ROLLUP).await.expect("second eval");

    // AgentProfile has no PartialEq; compare observable output via serialization.
    let a = serde_json::to_value(&first).expect("serialize first");
    let b = serde_json::to_value(&second).expect("serialize second");
    assert_eq!(a, b, "two evals of the same source must be identical");
}

#[tokio::test(flavor = "current_thread")]
async fn date_now_is_blocked() {
    let src = r#"
        import { defineAgent } from "ship";
        const stamp = Date.now();
        defineAgent({ agent: { id: "x", name: "x", version: String(stamp) } });
    "#;
    let err = resolve_agents(src)
        .await
        .expect_err("Date.now() must fail the eval");
    let msg = format!("{err:#}");
    assert!(
        msg.contains("Date.now()"),
        "error must name the blocked call: {msg}"
    );
}

#[tokio::test(flavor = "current_thread")]
async fn math_random_is_blocked() {
    let src = r#"
        import { defineAgent } from "ship";
        defineAgent({ agent: { id: "x", name: "x", version: String(Math.random()) } });
    "#;
    let err = resolve_agents(src)
        .await
        .expect_err("Math.random() must fail the eval");
    let msg = format!("{err:#}");
    assert!(
        msg.contains("Math.random()"),
        "error must name the blocked call: {msg}"
    );
}

#[tokio::test(flavor = "current_thread")]
async fn parameterized_date_is_allowed() {
    // The guard blocks the non-deterministic forms only; a fixed timestamp is
    // fine, so deterministic config that touches Date still evaluates.
    let src = r#"
        import { defineAgent } from "ship";
        const built = new Date("2020-01-01T00:00:00Z").toISOString();
        defineAgent({ agent: { id: "x", name: "x", version: built } });
    "#;
    let profiles = resolve_agents(src)
        .await
        .expect("parameterized Date must be allowed");
    assert_eq!(profiles.len(), 1);
    assert_eq!(
        profiles[0].agent.version.as_deref(),
        Some("2020-01-01T00:00:00.000Z")
    );
}

#[tokio::test(flavor = "current_thread")]
async fn non_ship_import_is_denied() {
    let src = r#"
        import { readFileSync } from "fs";
        import { defineAgent } from "ship";
        defineAgent({ agent: { id: "x", name: "x" } });
    "#;
    let err = resolve_agents(src)
        .await
        .expect_err("importing anything but ship must fail");
    let msg = format!("{err:#}");
    assert!(
        msg.contains("import denied") || msg.contains("fs"),
        "error must indicate the import was denied: {msg}"
    );
}

#[tokio::test(flavor = "current_thread")]
async fn real_typescript_is_stripped_and_evaluated() {
    // Fails to evaluate as-is (V8 rejects `import type`, `: Partial<…>`,
    // `satisfies`, `as const`); only the strip makes it JS-valid.
    let profiles = resolve_agents(TYPED)
        .await
        .expect("typed TypeScript config evaluates after strip");

    let ids: Vec<&str> = profiles.iter().map(|p| p.agent.id.as_str()).collect();
    assert_eq!(ids, ["alpha", "beta"], "two typed defineAgent calls");
    // The spread `base` (annotated `Partial<AgentProfile>`) survives the strip.
    assert_eq!(profiles[0].capabilities[0].domain, "workspace");
    assert_eq!(profiles[1].capabilities[0].domain, "workspace");
}

#[tokio::test(flavor = "current_thread")]
async fn injected_globals_land_and_locals_override() {
    let globals = serde_json::json!({
        "agentDefaults": {
            "capabilities": [{ "domain": "workspace" }],
            "target_agents": ["mesh"],
        }
    });
    let profiles = resolve_agents_with_globals(GLOBALS, globals)
        .await
        .expect("config with injected globals evaluates");

    assert_eq!(profiles.len(), 2);
    // Injected default lands on every agent.
    assert_eq!(profiles[0].capabilities[0].domain, "workspace");
    assert_eq!(profiles[1].capabilities[0].domain, "workspace");
    // The default applies where the agent didn't override it…
    assert_eq!(profiles[0].target_agents, ["mesh"]);
    // …and a per-agent field wins over the injected default.
    assert_eq!(profiles[1].target_agents, ["local-only"]);
}

#[tokio::test(flavor = "current_thread")]
async fn globals_eval_is_deterministic() {
    let globals = serde_json::json!({
        "agentDefaults": { "capabilities": [{ "domain": "workspace" }] }
    });
    let first = resolve_agents_with_globals(GLOBALS, globals.clone())
        .await
        .expect("first eval");
    let second = resolve_agents_with_globals(GLOBALS, globals)
        .await
        .expect("second eval");

    // Same (source, globals) → byte-identical output.
    let a = serde_json::to_string(&first).expect("serialize first");
    let b = serde_json::to_string(&second).expect("serialize second");
    assert_eq!(a, b, "two evals of the same source + globals must be identical");
}
