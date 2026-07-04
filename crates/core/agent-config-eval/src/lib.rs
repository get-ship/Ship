//! Evaluate a `ship.ts` agent config — authored with `defineAgent` — in a
//! locked-down ship-js isolate and harvest the result as
//! [`compiler::AgentProfile`]s.
//!
//! # The boundary
//!
//! `ship-js` is a standalone JS engine with **zero** knowledge of ship's config
//! types. THIS crate supplies everything ship-specific — the TypeScript strip
//! ([`strip_types`]), the `@ship/sdk` stub ([`SDK_STUB`]), and the
//! [`AgentProfile`] harvest. The engine only runs a closed-world module and
//! returns the values it emits. The SDK is a *string parameter* to the engine,
//! never vendored into it.
//!
//! # TypeScript in, JavaScript to the engine
//!
//! `resolve_agents` accepts real TypeScript. [`strip_types`] runs a one-shot oxc
//! transform (type annotations, `import type`, `satisfies`/`as`, generics out)
//! producing equivalent JavaScript, which is what the engine evaluates. It is a
//! pure strip — value imports are preserved even when unused, and no ES
//! downleveling happens — so runtime semantics are unchanged.
//!
//! # The closed world
//!
//! - **Imports:** ship-js's deny-by-default loader resolves only `"ship"`, which
//!   it serves from the SDK module. Any other import fails to load.
//! - **Host wire:** [`DenyAll`] refuses every `ship_call` — config eval performs
//!   no side effects.
//! - **Determinism:** the stub reinstates a guard at module top level — which
//!   runs before the entry module's body, since ES modules evaluate their
//!   imports first — that blocks `Date.now`, `Math.random`, and zero-arg
//!   `new Date()`. The guard lives here (ship-supplied), not in the engine.
//!
//! # Injected globals — composition without a live fetch
//!
//! The runtime can pre-fetch a user's global defaults and hand them to the eval
//! as a plain value via [`resolve_agents_with_globals`]. The value is baked into
//! the SDK module as an exported `globals` object before any user code runs — no
//! fs/net/RPC — so the closed world and determinism hold: the globals are part
//! of the input. A `ship.ts` composes them with
//! `defineAgent({ ...globals.agentDefaults, agent: { id, name: id } })`.

use std::path::Path;
use std::sync::Arc;

use anyhow::Context;
use compiler::AgentProfile;
use oxc_allocator::Allocator;
use oxc_codegen::Codegen;
use oxc_parser::Parser;
use oxc_semantic::SemanticBuilder;
use oxc_span::SourceType;
use oxc_transformer::{TransformOptions, Transformer};
use ship_js::{Dispatch, DispatchError};

/// Strip TypeScript types from `source`, returning equivalent JavaScript.
///
/// A pure, one-shot oxc transform: type annotations, `import type`,
/// `satisfies`/`as` casts, and generic parameters are removed, but runtime
/// semantics are preserved. `only_remove_type_imports` keeps every *value*
/// import even when it is unused, so a side-effecting (or denied) import still
/// reaches the loader instead of being elided. The default (ESNext) target means
/// no ES downleveling — only types come out. Comments are dropped.
pub fn strip_types(source: &str) -> anyhow::Result<String> {
    let allocator = Allocator::default();
    // `ship.ts` — non-JSX TypeScript. `<T>x` is a type assertion here, not JSX.
    let source_type = SourceType::ts();
    let path = Path::new("ship.ts");

    let parsed = Parser::new(&allocator, source, source_type).parse();
    if !parsed.errors.is_empty() {
        let msg = parsed
            .errors
            .iter()
            .map(|e| e.to_string())
            .collect::<Vec<_>>()
            .join("; ");
        anyhow::bail!("TypeScript parse error: {msg}");
    }
    let mut program = parsed.program;

    let scoping = SemanticBuilder::new()
        .build(&program)
        .semantic
        .into_scoping();

    let mut options = TransformOptions::default();
    // Pure type-strip: never elide unused value imports (a denied `import` must
    // still reach the loader), only erase `import type`.
    options.typescript.only_remove_type_imports = true;

    let ret = Transformer::new(&allocator, path, &options).build_with_scoping(scoping, &mut program);
    if !ret.errors.is_empty() {
        let msg = ret
            .errors
            .iter()
            .map(|e| e.to_string())
            .collect::<Vec<_>>()
            .join("; ");
        anyhow::bail!("TypeScript strip error: {msg}");
    }

    Ok(Codegen::new().build(&program).code)
}

/// The ship-supplied `@ship/sdk` module body — served to `import … from "ship"`.
///
/// It has two jobs, both at module top level so they run before the entry
/// module's body:
///
/// 1. Install the determinism guard.
/// 2. Export `defineAgent`, the collector: each call emits one config into the
///    engine's collector registry, harvested back on the Rust side after eval.
///
/// The injected `globals` export is prepended by [`sdk_module`] — see there.
pub const SDK_STUB: &str = r#"
// --- determinism guard: config eval must be pure -------------------------
// Runs at SDK-module load, before the entry module's body (ES modules evaluate
// imports first), so authored config never observes an un-guarded clock/RNG.
const __block = (what) => () => {
  throw new Error(what + " is blocked in ship config eval — config must be deterministic");
};
Math.random = __block("Math.random()");
Date.now = __block("Date.now()");
{
  const _Date = Date;
  globalThis.Date = new Proxy(_Date, {
    construct(target, args) {
      if (args.length === 0) {
        throw new Error("new Date() is blocked in ship config eval — config must be deterministic");
      }
      return Reflect.construct(target, args);
    },
    apply() {
      throw new Error("Date() is blocked in ship config eval — config must be deterministic");
    },
  });
}

// --- defineAgent: the collector ------------------------------------------
// Each call harvests one agent definition into the engine's collector registry
// (op_emit), read back on the Rust side once the module settles. N calls — e.g.
// in a loop — collect N entries, in call order.
export function defineAgent(config) {
  Deno.core.ops.op_emit(config);
  return config;
}
"#;

/// Build the served `@ship/sdk` module: the injected `globals` export followed
/// by [`SDK_STUB`].
///
/// `globals` is a plain data literal (JSON is valid JS expression syntax) baked
/// in before any user code runs — no fs/net/RPC, so the closed world and
/// determinism hold. Config reads it via `import { globals } from "ship"`.
fn sdk_module(globals: &serde_json::Value) -> anyhow::Result<String> {
    let json = serde_json::to_string(globals).context("serialize injected globals for SDK stub")?;
    Ok(format!("export const globals = {json};\n{SDK_STUB}"))
}

/// A host dispatch that denies every op: config eval has no effects. A denied
/// op surfaces to the program as `{ error }` DATA (the engine's contract), so a
/// config that reaches for the wire keeps running but changes nothing.
struct DenyAll;

#[async_trait::async_trait]
impl Dispatch for DenyAll {
    async fn call(
        &self,
        op: &str,
        _args: serde_json::Value,
    ) -> Result<serde_json::Value, DispatchError> {
        Err(DispatchError::Terminal(format!(
            "host op `{op}` is denied during config eval"
        )))
    }
}

/// Evaluate a `ship.ts` config module and harvest its `defineAgent` calls as
/// [`AgentProfile`]s, in call order.
///
/// `source` is real TypeScript — [`strip_types`] lowers it to JavaScript before
/// eval. N `defineAgent(…)` calls — including inside a loop — roll up to N
/// profiles. Thin wrapper over [`resolve_agents_with_globals`] with no injected
/// globals.
pub async fn resolve_agents(source: &str) -> anyhow::Result<Vec<AgentProfile>> {
    resolve_agents_with_globals(source, serde_json::Value::Null).await
}

/// Like [`resolve_agents`], but with pre-fetched global defaults injected into
/// the eval as an exported `globals` object (see the crate docs). `globals` is
/// part of the closed-world input — no live fetch — so the eval stays
/// deterministic.
pub async fn resolve_agents_with_globals(
    source: &str,
    globals: serde_json::Value,
) -> anyhow::Result<Vec<AgentProfile>> {
    let js = strip_types(source).context("TypeScript type-strip failed")?;
    let sdk = sdk_module(&globals)?;

    let dispatch: Arc<dyn Dispatch> = Arc::new(DenyAll);
    let emitted = ship_js::deno::run_module(dispatch, sdk, js)
        .await
        .context("config eval failed")?;

    let mut profiles = Vec::with_capacity(emitted.len());
    for (i, value) in emitted.into_iter().enumerate() {
        let profile: AgentProfile = serde_json::from_value(value)
            .with_context(|| format!("defineAgent call #{i} did not match AgentProfile"))?;
        profiles.push(profile);
    }
    Ok(profiles)
}
