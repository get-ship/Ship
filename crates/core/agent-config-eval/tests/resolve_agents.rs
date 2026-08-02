//! ARC A5: `defineAgent` is retired. Agent nouns load from `agents/*.jsonc`.
//! This crate still evaluates code-noun declare-forms (pipelines/triggers/
//! views/collections + defineConfig for dir pointers / permission presets).

use agent_config_eval::{resolve_agents, resolve_declarations_with_globals};

#[tokio::test(flavor = "current_thread")]
async fn define_agent_is_rejected_loudly() {
    let src = r#"
        import { defineAgent } from "ship";
        defineAgent({ agent: { id: "x", name: "x" } });
    "#;
    let err = resolve_agents(src)
        .await
        .expect_err("defineAgent must fail closed after ARC A5");
    let msg = format!("{err:#}");
    assert!(
        msg.contains("defineAgent") && msg.contains("retired"),
        "expected ARC A5 retirement message, got: {msg}"
    );
}

#[tokio::test(flavor = "current_thread")]
async fn define_pipeline_still_harvests() {
    let src = r#"
        import { definePipeline } from "ship";
        definePipeline({
          name: "ci",
          steps: [{ id: "build", after: [], step_specs: [] }],
          auto_merge: false,
        });
    "#;
    let decls = resolve_declarations_with_globals(src, serde_json::Value::Null)
        .await
        .expect("pipeline-only eval must succeed");
    assert!(decls.agents.is_empty());
    assert_eq!(decls.pipelines.len(), 1);
    assert_eq!(decls.pipelines[0].name, "ci");
}

#[tokio::test(flavor = "current_thread")]
async fn date_now_is_blocked() {
    let src = r#"
        import { definePipeline } from "ship";
        const stamp = Date.now();
        definePipeline({
          name: String(stamp),
          steps: [],
          auto_merge: false,
        });
    "#;
    let err = resolve_declarations_with_globals(src, serde_json::Value::Null)
        .await
        .expect_err("Date.now must be blocked");
    let msg = format!("{err:#}");
    assert!(
        msg.contains("Date") || msg.contains("determinism") || msg.contains("blocked") || msg.contains("config eval"),
        "unexpected error: {msg}"
    );
}
