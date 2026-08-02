//! V8 eval fan-out for the project-shape resolver.
//!
//! The resolver's TS surface is multi-file in Phase 3 (`stack.config.ts` +
//! `agents.ts` + per-domain modules — the orchestrator lean from
//! `design-interview-2026-07-05.md:97-119`). Phase 2 takes the seam from
//! the shipd ingest — one source string per `imported_modules` entry — and
//! the closed-world loader still admits ONLY `"ship"`; project-relative
//! `.ts` imports are a Phase-3 ship-js change (R10 in
//! `census-phase2-resolver.md`).
//!
//! ## The `!Send` boundary (R2)
//!
//! `agent_config_eval::resolve_agents_with_globals` is async + `!Send`-
//! isolated (the `deno_core::JsRuntime` it pulls in is `!Send`). The V8
//! future CANNOT be `await`ed directly from a multi-threaded tokio
//! runtime — the future is not `Send` so the executor refuses to move it
//! across worker threads. The seam shipd uses today
//! (`apps/shipd/src/ship_runtime/tasks.rs:285-304`) is:
//!
//! 1. `tokio::task::spawn_blocking(move || { ... })` — moves the work to
//!    the blocking thread pool.
//! 2. INSIDE the closure, build a fresh `tokio::runtime::Builder::new_current_thread()`
//!    and `.block_on(resolve_agents_with_globals(...))` — the V8 isolate
//!    lives on that one thread, so the future never has to be `Send`.
//!
//! We mirror that seam here. The result is `Send` (a `Vec<AgentProfile>`
//! is plain data) and flows back to the multi-threaded executor without
//! re-crossing the V8 boundary.
//!
//! ## Closed world (the contract we depend on)
//!
//! `agent-config-eval` reuses the `ship-js` engine with the closed-world
//! deny-by-default module loader (`ship-js/src/deno/module_loader.rs`):
//! only `"ship"` resolves. Any other import — `node:fs`, `http`, even a
//! project-relative `./agents.ts` — fails to load. So Phase 2's TS path
//! is exactly the single-`stack.config.ts` semantics shipd's ingest has today:
//! one source string, one V8 eval, N `defineAgent` calls rolled up to N
//! profiles.
//!
//! When Phase 3 extends the loader to admit project-relative `.ts`
//! imports (R10), the fan-out grows to N evals — one per `imported_modules`
//! entry, merged in declared order. The function signature below takes
//! `&[ModuleDescriptor]` so the Phase-3 call site is the same shape; the
//! Phase-2 implementation simply iterates with a single entry.

use agent_config_eval::Declarations;
use anyhow::{Context, Result};
use serde_json::Value;
use stack_model::ModuleDescriptor;

/// Merge one module's [`Declarations`] into the running total, in
/// declared order — every noun vector appends, per-noun call order
/// within a module preserved.
fn merge_declarations(all: &mut Declarations, module: Declarations) {
    all.agents.extend(module.agents);
    all.mcp_servers.extend(module.mcp_servers);
    all.skills.extend(module.skills);
    all.rules.extend(module.rules);
    all.pipelines.extend(module.pipelines);
    all.triggers.extend(module.triggers);
    all.collections.extend(module.collections);
    all.views.extend(module.views);
    all.config.extend(module.config);
}

/// Evaluate every TS module in `imported_modules` and return the merged
/// declare-form roll-up across all of them, in declared order — every
/// noun a `stack.config.ts` can declare (`defineAgent`, `defineMcpServer`,
/// `defineSkill`, `definePipeline`, `defineTrigger`, `defineCollection`,
/// `defineRule`), not just agents (format-migration Wave 3).
///
/// The default `globals` (operator pre-fetched defaults, baked into the
/// SDK module as a closed-world input) is the same value
/// `agent-config-eval::resolve_declarations_with_globals` accepts. The closed
/// world holds: `globals` is part of the eval input, no live fetch.
///
/// The `!Send` V8 boundary is crossed per-module inside this fn — each
/// module's eval runs in its own `spawn_blocking` so a misbehaving module
/// (a panic, an OOM in V8) is contained and surfaces as a logged error
/// without aborting the fan-out. A failure on one module is logged and
/// the eval continues; the merge returns whatever the successful evals
/// produced. A failure on the only module is returned as the only error.
///
/// `imported_modules` is the same `Vec<ModuleDescriptor>` the descriptor
/// carries in `ResolvedConfig::imported_modules` (and in
/// `ExtensionLibrary::imported_modules`). An empty slice is the
/// "no TS modules" case — the resolver on a `.jsonc`-only project; this
/// fn returns an empty [`Declarations`].
pub async fn resolve_ts_files(
    imported_modules: &[ModuleDescriptor],
    globals: &Value,
) -> Result<Declarations> {
    let mut all = Declarations::default();
    if imported_modules.is_empty() {
        return Ok(all);
    }

    for module in imported_modules {
        match eval_one_module(module, globals).await {
            Ok(declarations) => merge_declarations(&mut all, declarations),
            Err(e) => {
                // Phase-2 contract: a single module's eval failure is
                // logged and the fan-out continues. The descriptor cache
                // stores what the successful evals produced; the next
                // pass over a fixed module re-runs and re-validates.
                tracing::warn!(
                    "ts_resolver: eval failed for module {}: {e}",
                    module.path.display()
                );
            }
        }
    }
    Ok(all)
}

/// Resolve a single TS module's declare-form calls, crossing the
/// `!Send` V8 boundary on a dedicated current-thread runtime.
///
/// This is the same `spawn_blocking` + `new_current_thread` pattern
/// shipd's ingest uses (`apps/shipd/src/ship_runtime/tasks.rs:295-300`).
/// The closure is `Send + 'static` (it owns the source + globals + sdk
/// path), the V8 future lives on the inner thread, and the resolved
/// `Declarations` is `Send` so it flows back to the multi-threaded
/// executor without re-crossing the V8 boundary.
async fn eval_one_module(module: &ModuleDescriptor, globals: &Value) -> Result<Declarations> {
    let source = module.source.clone();
    let globals = globals.clone();

    // Move the V8 work onto the blocking thread pool; inside the closure
    // we mint a fresh current-thread runtime for the eval.
    let declarations = tokio::task::spawn_blocking(move || -> Result<Declarations> {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .context("ts_resolver: failed to build current-thread runtime for V8 eval")?;
        rt.block_on(agent_config_eval::resolve_declarations_with_globals(
            &source, globals,
        ))
    })
    .await
    .context("ts_resolver: V8 eval task join failed (panic?)")??;

    Ok(declarations)
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn empty_module(path: &str, source: &str) -> ModuleDescriptor {
        ModuleDescriptor {
            path: PathBuf::from(path),
            source: source.to_string(),
            content_hash: String::new(),
            exports: Vec::new(),
        }
    }

    #[tokio::test(flavor = "current_thread")]
    async fn empty_module_list_yields_no_declarations() {
        let declarations = resolve_ts_files(&[], &Value::Null).await.unwrap();
        assert!(declarations.agents.is_empty());
        assert!(declarations.mcp_servers.is_empty());
        assert!(declarations.skills.is_empty());
        assert!(declarations.rules.is_empty());
        assert!(declarations.pipelines.is_empty());
        assert!(declarations.triggers.is_empty());
        assert!(declarations.collections.is_empty());
    }

    #[tokio::test(flavor = "current_thread")]
    async fn define_agent_module_is_skipped_after_a5() {
        // ARC A5: defineAgent fails harvest; resolve_ts_files logs and
        // continues with empty agents (no silent dual path).
        let module = empty_module(
            "stack.config.ts",
            r#"
import { defineAgent } from "ship";

defineAgent({
  agent: { id: "demo", name: "Demo" },
});
"#,
        );
        let declarations = resolve_ts_files(&[module], &Value::Null).await.unwrap();
        assert!(
            declarations.agents.is_empty(),
            "defineAgent must not harvest agents after ARC A5"
        );
    }

    #[tokio::test(flavor = "current_thread")]
    async fn code_noun_eval_is_deterministic_across_two_runs() {
        let source = r#"
import { definePipeline } from "ship";

definePipeline({
  name: "ci",
  steps: [{ id: "build", after: [], step_specs: [] }],
  auto_merge: false,
});
"#;
        let m1 = empty_module("stack.config.ts", source);
        let m2 = empty_module("stack.config.ts", source);
        let d1 = resolve_ts_files(&[m1], &Value::Null).await.unwrap();
        let d2 = resolve_ts_files(&[m2], &Value::Null).await.unwrap();
        assert_eq!(d1.pipelines.len(), 1);
        assert_eq!(d2.pipelines.len(), 1);
        assert_eq!(d1.pipelines[0].name, d2.pipelines[0].name);
    }

    #[tokio::test(flavor = "current_thread")]
    async fn multiple_modules_merge_code_nouns_in_declared_order() {
        let m1 = empty_module(
            "a.ts",
            r#"
import { definePipeline } from "ship";
definePipeline({ name: "a1", steps: [], auto_merge: false });
"#,
        );
        let m2 = empty_module(
            "b.ts",
            r#"
import { definePipeline } from "ship";
definePipeline({ name: "b1", steps: [], auto_merge: false });
definePipeline({ name: "b2", steps: [], auto_merge: false });
"#,
        );
        let declarations = resolve_ts_files(&[m1, m2], &Value::Null).await.unwrap();
        let names: Vec<&str> = declarations
            .pipelines
            .iter()
            .map(|p| p.name.as_str())
            .collect();
        assert_eq!(names, vec!["a1", "b1", "b2"]);
    }

    /// ARC A5: agent nouns retired from TS; code nouns + defineConfig
    /// (permissions / dir pointers) still harvest.
    #[tokio::test(flavor = "current_thread")]
    async fn every_code_declare_form_lands_in_its_noun_field() {
        let module = empty_module(
            "stack.config.ts",
            r#"
import {
  definePipeline,
  defineTrigger,
  defineCollection,
  defineView,
  defineConfig,
} from "ship";

defineConfig({
  dependencies: {
    "github.com/better-auth/skills": "main",
  },
  agents: {
    permissions: {
      presets: {
        "ship-standard": {
          default_mode: "default",
          tools_allow: ["Read"],
          tools_deny: [],
        },
      },
    },
  },
});

definePipeline({
  name: "ci",
  steps: [{ id: "build", after: [], step_specs: [] }],
  auto_merge: false,
});

defineTrigger({
  id: "on-push",
  trigger: { event: "git.push" },
  actions: [{ type: "event.emit" }],
});

defineCollection({
  id: "notes",
  name: "Notes",
  fields: [{ name: "title", type: "string", required: true, values: [] }],
});

defineView({
  route: "/hello",
  component_ref: "Hello.sui",
});
"#,
        );
        let declarations = resolve_ts_files(&[module], &Value::Null).await.unwrap();

        assert!(declarations.agents.is_empty());
        assert!(declarations.mcp_servers.is_empty());
        assert_eq!(declarations.config.len(), 1);
        let cfg = &declarations.config[0];
        let agents = cfg.agents.as_ref().expect("agents block in defineConfig");
        assert!(agents.mcp.is_empty());
        assert!(agents.permissions.is_some());
        let dep = cfg
            .dependencies
            .get("github.com/better-auth/skills")
            .expect("defineConfig dependencies harvested");
        match dep {
            stack_model::manifest::ManifestDepValue::Version(version) => {
                assert_eq!(version, "main")
            }
            stack_model::manifest::ManifestDepValue::Full(_) => {
                panic!("expected shorthand dependency")
            }
        }

        assert_eq!(declarations.pipelines.len(), 1);
        assert_eq!(declarations.pipelines[0].name, "ci");

        assert_eq!(declarations.triggers.len(), 1);
        assert_eq!(declarations.triggers[0].id, "on-push");

        assert_eq!(declarations.collections.len(), 1);
        assert_eq!(declarations.collections[0].id, "notes");

        // Skills + rules are no longer in the TS harvest; they live
        // on disk and land via the resolver's on-disk walks.
        assert!(declarations.skills.is_empty());
        assert!(declarations.rules.is_empty());

        assert_eq!(declarations.views.len(), 1);
        assert_eq!(declarations.views[0].route, "/hello");
        assert_eq!(declarations.views[0].component_ref, "Hello.sui");
    }
}
