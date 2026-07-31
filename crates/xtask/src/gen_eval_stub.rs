//! `gen-eval-stub` — regenerate `agent-config-eval`'s sandboxed-eval SDK
//! stub from `@ship/stack`'s noun shapes (format-migration Wave 3,
//! `decision-morning-answers.md` §1: the isolate's declare-forms are
//! INJECTED BY THE RUNTIME and generated from the stack shapes with a
//! drift gate — not hand-maintained).
//!
//! Output: `crates/core/agent-config-eval/src/generated_stub.rs` (committed;
//! un-ignored so drift surfaces in PRs, same convention as `gen-sdk`'s
//! `packages/sdk/src/manifests/`). It carries two things:
//!
//! 1. `SDK_STUB` — the JS text served to the `"ship"` import. Every
//!    declare-form but `defineRule` is a generic collector: `__emit(kind,
//!    config)` tags the value with its noun kind and forwards it to
//!    `Deno.core.ops.op_emit`, so the untyped JS side needs no per-noun
//!    logic — only the noun LIST here is derived from the stack shapes.
//! 2. One `pub const <NOUN>_FIELDS: &[&str]` per fingerprinted stack type —
//!    its sorted, exported field names. `ts_resolver.rs`'s hand-written
//!    harvest structs declare a matching const beside their
//!    `#[derive(Deserialize)]` struct; a unit test there asserts the two
//!    stay equal. That closes the loop this file's `--check` opens: a
//!    stack shape change that isn't regenerated here fails CI; a
//!    regeneration that isn't reflected in the harvest struct fails
//!    `cargo test -p space-resolver`.
//!
//! `defineRule` is EXCLUDED from stack-derived fingerprinting.
//! `decision-morning-answers.md` §2 rules the rules declare-form down to
//! the thinnest possible shape (id + content string + apply scope) —
//! deliberately NOT stack's `RuleDef` (frontmatter + body), which the
//! ruling itself flags as needing realignment in a later wave. Until
//! that lands, the rules shape here is hand-specified, not generated;
//! see `ts_resolver.rs`'s `RuleDeclare`.
//!
//! `defineView` is ALSO EXCLUDED from stack-derived fingerprinting
//! (Phase-4 J2) for the same J1-provisional-pending-sail/deck-alignment
//! reason documented in `decision-views-deck-boundary.md`. The shape
//! here is the thinnest one that round-trips through J1's
//! `ViewDescriptor`: a local `route` plus a `component_ref`. The
//! per-extension `prefix` is set by the roll-up, not by the author.
//!
//! Usage:
//!   cargo xtask gen-eval-stub          # regenerate the committed file
//!   cargo xtask gen-eval-stub --check  # diff vs on-disk; exit 1 on drift

use std::{
    env, fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, bail};
use oxc_allocator::Allocator;
use oxc_ast::ast::{Declaration, Statement, TSSignature, TSType};
use oxc_parser::Parser;
use oxc_span::SourceType;

const OUT_PATH: &str = "crates/core/agent-config-eval/src/generated_stub.rs";

/// One stack noun shape to fingerprint: the TS file it lives in (relative
/// to `@ship/stack/src/`), the exported type alias name, and the Rust
/// const name the field-name list is emitted under.
struct NounType {
    file: &'static str,
    type_name: &'static str,
    const_name: &'static str,
}

const NOUN_TYPES: &[NounType] = &[
    NounType { file: "mcp-servers.ts", type_name: "McpServerDef", const_name: "MCP_SERVER_DEF_FIELDS" },
    NounType { file: "pipelines.ts", type_name: "PipelineDef", const_name: "PIPELINE_DEF_FIELDS" },
    NounType { file: "pipelines.ts", type_name: "PipelineStep", const_name: "PIPELINE_STEP_FIELDS" },
    NounType { file: "pipelines.ts", type_name: "PipelineGate", const_name: "PIPELINE_GATE_FIELDS" },
    NounType {
        file: "pipelines.ts",
        type_name: "PipelineStepSpec",
        const_name: "PIPELINE_STEP_SPEC_FIELDS",
    },
    NounType { file: "triggers.ts", type_name: "TriggerDef", const_name: "TRIGGER_DEF_FIELDS" },
    NounType {
        file: "triggers.ts",
        type_name: "TriggerEventMatch",
        const_name: "TRIGGER_EVENT_MATCH_FIELDS",
    },
    NounType { file: "triggers.ts", type_name: "TriggerAction", const_name: "TRIGGER_ACTION_FIELDS" },
    NounType {
        file: "triggers.ts",
        type_name: "TriggerSourceWebhook",
        const_name: "TRIGGER_SOURCE_WEBHOOK_FIELDS",
    },
    NounType {
        file: "triggers.ts",
        type_name: "TriggerSourceFile",
        const_name: "TRIGGER_SOURCE_FILE_FIELDS",
    },
    NounType {
        file: "triggers.ts",
        type_name: "TriggerSourceCron",
        const_name: "TRIGGER_SOURCE_CRON_FIELDS",
    },
    NounType {
        file: "ship-data.ts",
        type_name: "CollectionFieldDef",
        const_name: "COLLECTION_FIELD_DEF_FIELDS",
    },
];

/// Declare-forms injected as `"ship"` module exports: (JS function name,
/// `__kind` discriminant tag). `defineAgent` is the pre-existing form
/// (Phase 0); `definePipeline` / `defineTrigger` / `defineCollection` /
/// `defineView` are the surviving Wave 3 + Phase-4 additions. Five
/// forms share one generic collector body — see [`render_sdk_stub`].
/// `defineSkill` + `defineMcpServer` are gone (slice 2c):
/// skills live on-disk under `defineConfig({ skills_dir })`,
/// and MCP server registrations live under
/// `defineConfig({ agents: { mcp: { … } } })`. `defineRule`
/// was retired earlier (the on-disk `rules/<id>.md` convention
/// also died with the rules-format cut — rule bodies now live on
/// the agent composition as `defineAgent({ rules: { inline } })`).
///
/// `defineView` is the Phase-4 J2 addition. The hand-specified
/// field set (no stack fingerprint) follows the same precedent
/// the slice-2c-closeout rules-shape precedent set per
/// `decision-morning-answers.md` §2: J1's `ViewDescriptor` is
/// PROVISIONAL pending sail/deck alignment
/// (`decision-views-deck-boundary.md`), so we don't bind a stack shape
/// that may pass through one more pass before the deck update lands.
/// The hand-specified field set is asserted stable in
/// `agent-config-eval/src/declarations.rs::drift_tests`.
const DECLARE_FORMS: &[(&str, &str)] = &[
    ("defineAgent", "agent"),
    ("definePipeline", "pipeline"),
    ("defineTrigger", "trigger"),
    ("defineCollection", "collection"),
    ("defineView", "view"),
];

pub fn run(repo_root: &Path, check: bool) -> Result<()> {
    let stack_src = stack_src_dir(repo_root)?;

    let mut fields = Vec::with_capacity(NOUN_TYPES.len());
    for noun in NOUN_TYPES {
        let names = extract_fields(&stack_src, noun.file, noun.type_name)
            .with_context(|| format!("fingerprinting {} in {}", noun.type_name, noun.file))?;
        fields.push((noun.const_name, names));
    }

    let rendered = render(&fields);
    let out_path = repo_root.join(OUT_PATH);

    if check {
        let existing = fs::read_to_string(&out_path).unwrap_or_default();
        if existing != rendered {
            eprintln!("gen-eval-stub: out of date: {}", out_path.display());
            eprintln!("Run `cargo xtask gen-eval-stub` and commit the result.");
            std::process::exit(1);
        }
        eprintln!("gen-eval-stub: up-to-date ({})", out_path.display());
        return Ok(());
    }

    if let Some(parent) = out_path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    fs::write(&out_path, &rendered).with_context(|| format!("write {}", out_path.display()))?;
    eprintln!("gen-eval-stub: wrote {}", out_path.display());
    Ok(())
}

/// Locate `@ship/stack`'s `src/` dir: `SHIP_STACK_DIR` env override, else
/// the sibling-repo default. Mirrors the `SHIP_PROTO_DIR` convention
/// (`apps/helmd/build.rs`) — the default only resolves when `ship` and
/// `ship-contracts` are sibling checkouts (broken at worktree depth; a
/// worktree run sets the env var).
fn stack_src_dir(repo_root: &Path) -> Result<PathBuf> {
    let dir = match env::var("SHIP_STACK_DIR") {
        Ok(p) => PathBuf::from(p),
        Err(_) => repo_root.join("../ship-contracts/packages/stack/src"),
    };
    if !dir.is_dir() {
        bail!(
            "@ship/stack src dir not found at {} (set SHIP_STACK_DIR to override, e.g. the \
             sibling repo's packages/stack/src)",
            dir.display()
        );
    }
    Ok(dir)
}

/// Parse `file` under `stack_src` and return the sorted, exported field
/// names of the `type <type_name> = { ... }` alias — a flat object
/// literal. Nested/union shapes are fingerprinted by naming their own
/// concrete member type in [`NOUN_TYPES`], not by recursing here.
fn extract_fields(stack_src: &Path, file: &str, type_name: &str) -> Result<Vec<String>> {
    let path = stack_src.join(file);
    let source = fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;

    let allocator = Allocator::default();
    let source_type = SourceType::ts();
    let parsed = Parser::new(&allocator, &source, source_type).parse();
    if !parsed.errors.is_empty() {
        let msg = parsed.errors.iter().map(|e| e.to_string()).collect::<Vec<_>>().join("; ");
        bail!("TypeScript parse error in {}: {msg}", path.display());
    }

    for stmt in &parsed.program.body {
        let Statement::ExportNamedDeclaration(export) = stmt else { continue };
        let Some(Declaration::TSTypeAliasDeclaration(alias)) = &export.declaration else {
            continue;
        };
        if alias.id.name.as_str() != type_name {
            continue;
        }
        let TSType::TSTypeLiteral(literal) = &alias.type_annotation else {
            bail!("`{type_name}` in {file} is not a flat object type literal — cannot fingerprint");
        };
        let mut names: Vec<String> = literal
            .members
            .iter()
            .filter_map(|member| match member {
                TSSignature::TSPropertySignature(prop) => {
                    prop.key.static_name().map(|n| n.into_owned())
                }
                _ => None,
            })
            .collect();
        names.sort();
        return Ok(names);
    }
    bail!("exported type alias `{type_name}` not found in {file}");
}

/// Render the committed generated file: header, `SDK_STUB`, then one
/// field-fingerprint const per fingerprinted noun.
fn render(fields: &[(&str, Vec<String>)]) -> String {
    let mut out = String::new();
    out.push_str(
        "// GENERATED by `cargo xtask gen-eval-stub` — DO NOT EDIT.\n\
         // Source: ship-contracts/packages/stack/src (McpServerDef, PipelineDef,\n\
         // PipelineStep, PipelineGate, PipelineStepSpec, TriggerDef, TriggerEventMatch,\n\
         // TriggerAction, TriggerSourceWebhook/File/Cron, CollectionFieldDef).\n\
         // Run `cargo xtask gen-eval-stub --check` to verify this file is current;\n\
         // `cargo xtask gen-eval-stub` to refresh after a stack shape change.\n\
         //\n\
         // Slice 2c close-out: SkillDef / SkillFrontmatter / SkillAssetRef retired\n\
         // — skills are on-disk `skills/<id>/SKILL.md` files walked from\n\
         // `defineConfig({ skills_dir })`, not a stack-fingerprinted declare-form.\n\
         //\n\
         // `defineView` is EXCLUDED from generation (Phase-4 J2) — J1's\n\
         // `ViewDescriptor` is PROVISIONAL pending sail/deck alignment\n\
         // (decision-views-deck-boundary.md). Its field set is asserted\n\
         // stable by a hand-written-against-itself unit test in\n\
         // `declarations.rs::drift_tests`.\n\n",
    );

    out.push_str("pub const SDK_STUB: &str = r#\"\n");
    out.push_str(&render_sdk_stub());
    out.push_str("\"#;\n");

    out.push_str(
        "\n/// Field fingerprints — sorted, exported field names lifted from each\n\
         /// stack noun shape at generation time. See the file header: a unit test\n\
         /// in `ts_resolver.rs` asserts its hand-written harvest structs declare\n\
         /// the same field sets.\n",
    );
    for (const_name, names) in fields {
        let quoted: Vec<String> = names.iter().map(|n| format!("{n:?}")).collect();
        out.push_str(&format!("pub const {const_name}: &[&str] = &[{}];\n", quoted.join(", ")));
    }
    out
}

/// The JS text served to `import … from "ship"`: the determinism guard
/// (unchanged from the Phase-0 hand-written stub) followed by one
/// generic-collector declare-form per [`DECLARE_FORMS`] entry.
fn render_sdk_stub() -> String {
    let mut js = String::new();
    js.push_str(
        r#"// --- determinism guard: config eval must be pure -------------------------
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

// --- declare-forms: the collector ------------------------------------------
// Each call harvests one declaration into the engine's collector registry
// (op_emit), tagged with its noun kind so the Rust harvest can route it.
// N calls — e.g. in a loop — collect N entries, in call order.
function __emit(kind, value) {
  Deno.core.ops.op_emit({ __kind: kind, value });
  return value;
}

"#,
    );
    for (fn_name, kind) in DECLARE_FORMS {
        js.push_str(&format!(
            "export function {fn_name}(config) {{\n  return __emit(\"{kind}\", config);\n}}\n"
        ));
    }
    js
}
