# sail-lsp

Language server for Sail `.set` single-file components. The editor
surface that ties [`tree-sitter-set`](../../grammars/tree-sitter-set/)
(parse foundation) to [`sail-compiler`](../sail-compiler/) (semantic
analyzer).

## Status: v0 scaffold

Wired:

- `initialize` / `initialized` / `shutdown` / `exit` lifecycle
- `textDocument/didOpen`, `textDocument/didChange`,
  `textDocument/didClose`
- Push diagnostics via `textDocument/publishDiagnostics` — every
  open/change runs `sail_compiler::compile_source` and republishes
- Diagnostic severity, code (`SAIL/E####`), source (`"sail"`), span,
  and `help` text propagated 1:1

Stubbed (capabilities advertised so editors wire keybindings; handlers
return `null`):

- `textDocument/hover`
- `textDocument/completion`
- `textDocument/codeAction`
- `textDocument/definition`

## Run

```bash
cargo build -p sail-lsp
./target/debug/sail-lsp     # speaks LSP over stdio
```

Wire into your editor as a stdio LSP server matching `*.set` files.
Logs go to stderr (set `RUST_LOG=sail_lsp=debug` if you've added a
`tracing` subscriber upstream — the binary doesn't install one in v0
because the channel is JSON-RPC over stdout and we'd rather keep the
default quiet).

## Tests

```bash
cargo test -p sail-lsp
```

`server::tests::server_initializes_and_shuts_down_cleanly` is the
end-to-end smoke test — drives a memory-paired `Connection`, sends
`initialize` → `initialized` → `shutdown` → `exit`, and asserts the
server thread exits `Ok`.

## Why `lsp-server` and not `tower-lsp`?

Three reasons:

1. **Consistency.** `crates/lsp` (the Ship LSP — different surface,
   different consumer, same workspace) already runs on `lsp-server` /
   `lsp-types`. One transport, one mental model.
2. **No async runtime.** `lsp-server` is sync + single-threaded.
   Editor sessions are bursty and the compiler is fast — there's no
   concurrency we'd unlock by pulling in `tokio` and `tower`.
3. **Smaller surface.** `tower-lsp`'s trait-object dispatch wins when
   you're implementing dozens of methods. We have four wired and four
   stubs — direct match-on-method is shorter than the trait would be.

Trade-off: when we want incremental tree-sitter reparse with parallel
analysis, the sync model becomes a ceiling. That's a v2 question.

## Tree-sitter integration shape

The `tree-sitter` Rust crate is wired and a `Parser` lives on
`document::Documents`, but **no language is set in v0** — the
`tree-sitter-set` parser is being scaffolded in a parallel grammar job
and is not yet generated. Consequence: every `parse(...)` call returns
`None`, and `DocumentState::tree` stays empty.

The shape is in place so the follow-up only has to:

1. Add `tree-sitter-set = { path = "../../grammars/tree-sitter-set" }`
   to `Cargo.toml`.
2. Call `parser.set_language(&tree_sitter_set::LANGUAGE.into())` in
   `Documents::new`.
3. Switch `TextDocumentSyncKind` from `FULL` to `INCREMENTAL` and
   replay each `TextDocumentContentChangeEvent` range through
   `Tree::edit` before calling `Parser::parse(text, prev_tree)` — the
   `prev_tree` argument is already threaded through `Documents::upsert`.

Diagnostics today don't depend on the tree — they go straight through
`sail_compiler::compile_source`, which owns the canonical parse +
analysis. The tree is for future hover, completion, code-action, and
definition handlers that need positional queries against the syntax.

## Architecture

```text
client (editor)            sail-lsp                       sail-compiler
─────────────────          ─────────────                  ─────────────
didOpen / didChange  ─►   document::Documents::upsert
                            │     ▲
                            ▼     │
                          tree-sitter (stubbed v0)
                            │
                            ▼
                          diagnostics::compile_diagnostics ──►  compile_source
                            │                                        │
                            ▼                                        ▼
                          publishDiagnostics  ◄──────────────  Vec<Diagnostic>
```

Compiler stays pure transformation. LSP is thin transport — no
caching, no debounce, no semantic logic of its own.

## Next milestone

1. Tree-sitter-set parser usable → set the language, store real
   `Tree`s, switch to `INCREMENTAL` sync.
2. Hover handler — query the tree at position, surface the matching
   declaration's signature from the compiler's analysis output.
3. Code actions — wire compiler-suggested fixes (the `help` field on
   diagnostics already carries the text) as quick-fix actions.
4. Definition — jump from a state/prop reference to its declaration.

The reactivity-aware diagnostics (R1–R6 from
`.artifacts/specs/sail/`) and framework manifest export are downstream
of those four.
