//! Evaluate a `ship.config.ts` config — authored with `defineAgent` and, per
//! format-migration Wave 3, the sibling declare-forms (`defineMcpServer`,
//! `defineSkill`, `definePipeline`, `defineTrigger`, `defineCollection`,
//! `defineRule`) — in a locked-down ship-js isolate and harvest the result
//! as [`Declarations`].
//!
//! # The boundary
//!
//! `ship-js` is a standalone JS engine with **zero** knowledge of ship's config
//! types. THIS crate supplies everything ship-specific — the TypeScript strip
//! ([`strip_types`]), the `@ship/sdk` stub ([`SDK_STUB`], GENERATED from
//! `@ship/stack`'s noun shapes — see [`generated_stub`] and
//! `cargo xtask gen-eval-stub`), and the [`declarations::harvest`]. The engine
//! only runs a closed-world module and returns the values it emits. The SDK is
//! a *string parameter* to the engine, never vendored into it.
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
//! of the input. A `ship.config.ts` composes them with
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

mod declarations;
mod generated_stub;

pub use declarations::Declarations;
pub use generated_stub::SDK_STUB;

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
    // `ship.config.ts` — non-JSX TypeScript. `<T>x` is a type assertion here, not JSX.
    let source_type = SourceType::ts();
    let path = Path::new("ship.config.ts");

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

    let ret =
        Transformer::new(&allocator, path, &options).build_with_scoping(scoping, &mut program);
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

/// Evaluate a `ship.config.ts` config module and harvest its `defineAgent` calls as
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
/// deterministic. Thin filter over [`resolve_declarations_with_globals`]'s
/// `.agents` — every other declare-form kind is still evaluated (the isolate
/// doesn't know which nouns a caller wants), just not returned here.
pub async fn resolve_agents_with_globals(
    source: &str,
    globals: serde_json::Value,
) -> anyhow::Result<Vec<AgentProfile>> {
    Ok(resolve_declarations_with_globals(source, globals)
        .await?
        .agents)
}

/// Evaluate a `ship.config.ts` config module and harvest every declare-form call
/// (`defineAgent`, `defineMcpServer`, `defineSkill`, `definePipeline`,
/// `defineTrigger`, `defineCollection`, `defineRule`) into a
/// [`Declarations`], in call order per noun.
///
/// `source` is real TypeScript — [`strip_types`] lowers it to JavaScript
/// before eval. `globals` is the same closed-world injection
/// [`resolve_agents_with_globals`] accepts.
pub async fn resolve_declarations_with_globals(
    source: &str,
    globals: serde_json::Value,
) -> anyhow::Result<Declarations> {
    let js = strip_types(source).context("TypeScript type-strip failed")?;
    let sdk = sdk_module(&globals)?;

    let dispatch: Arc<dyn Dispatch> = Arc::new(DenyAll);
    let emitted = ship_js::deno::run_module(dispatch, sdk, js)
        .await
        .context("config eval failed")?;

    let mut declarations = Declarations::default();
    for (i, value) in emitted.into_iter().enumerate() {
        declarations::harvest(&mut declarations, i, value)?;
    }
    Ok(declarations)
}
