//! The linchpin: a `ship.ts` authored with `defineAgent`, evaluated in a
//! locked-down ship-js isolate, produces `compiler::AgentProfile`s — and a
//! program defining N agents in a loop rolls up to N profiles.

use agent_config_eval::resolve_agents;

/// A real `ship.ts` that defines agents in a `for` loop.
const ROLLUP: &str = include_str!("fixtures/rollup.ship.ts");

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
