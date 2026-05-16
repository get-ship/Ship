# SUI — Agent Surfaces

The set of programmatic surfaces an agent uses to author, diagnose, and
exercise SUI views or components. Three planes:

| Plane | Use it for | Lives in |
|---|---|---|
| **MCP tools** | Live operations against an active project (scaffold, edit, mount, snapshot, diagnose) | `apps/mcp/src/server/mod.rs`, `apps/mcp/src/tools/*.rs` |
| **LSP CLI** | One-shot diagnostics / hover / dependencies on a single `.sui` file from any shell | `packages/sui-lsp/src/cli.ts` |
| **`@ship/sui-eval`** | Regression-quality scoring of agent-produced SUI against a behavioural spec | `packages/sui-eval/src/index.ts` |

Read this alongside `SKILL.md` (the four-layer overview) and
`agent-authoring.md` (the do-now checklist for shipping a view in one
turn).

---

## 1. MCP tools

The `ship` MCP server registers 47 tools today. The ones relevant to
SUI authoring fall into seven groups; the rest (mesh, jobs, doc store,
artifacts, dispatch) are platform-level and documented in
`docs/ship-platform/`.

### 1.1 SUI / view authoring

| Tool | Signature | Use |
|---|---|---|
| `mcp__ship__scaffold_view` | `{ name, path?, layout? }` | Emit a fresh `views/<name>/` skeleton (`page.sui`, `controller.ts`, `manifest.jsonc`). Mirrors `node scripts/scaffold-view.mjs`. |
| `mcp__ship__sui_diagnostics` | `{ file }` | Parse + analyse a `.sui` file; returns `[{line, col, severity, message}]`. The first call any agent should make after writing a template. |
| `mcp__ship__sui_hover` | `{ file, line, col }` | Hover info at a cursor — prop type, directive doc, signal kind. |
| `mcp__ship__sui_dependencies` | `{ file }` | What `<sui-*>` tags, signals, and routes a template references — for impact analysis before edits. |
| `mcp__ship__view_compile_check` | `{ file }` | Full compile pass; returns `{ ok, errors, warnings }`. More expensive than `sui_diagnostics` but catches lowering-level issues. |
| `mcp__ship__get_view_errors` | `{ view }` | Latest runtime error stream for a view session (post-mount errors, not compile errors). |

### 1.2 View source I/O

| Tool | Signature | Use |
|---|---|---|
| `mcp__ship__view_read_source` | `{ view, file? }` | Read one file (or the whole view dir). Honours `_components/` layout. |
| `mcp__ship__view_write_source` | `{ view, file, contents }` | Write a file. Triggers daemon recompile + SSE notify. |
| `mcp__ship__view_list_siblings` | `{ view }` | Sibling components inside `_components/`, plus the parent's `page.sui` if applicable. |
| `mcp__ship__view_list_snapshots` | `{ view }` | List timeline snapshots — every save creates one. |
| `mcp__ship__view_restore_snapshot` | `{ view, snapshot_id }` | Revert to a snapshot — non-destructive (current state becomes a new snapshot first). |

### 1.3 View runtime control

For when the agent wants to mount a view in a live shell, then poke at
it (set a signal, patch HTML, capture state).

| Tool | Signature | Use |
|---|---|---|
| `mcp__ship__open_view` | `{ view, route? }` | Mount a view in the agent's session shell. Returns a `session_id`. |
| `mcp__ship__patch_view` | `{ session_id, selector, html }` | Replace the matched subtree's HTML (morph-style). |
| `mcp__ship__append_view` | `{ session_id, selector, html }` | Append HTML at a selector. |
| `mcp__ship__signal_view` | `{ session_id, name, value }` | Set a reactive signal — same effect as the user would via UI. |
| `mcp__ship__close_view` | `{ session_id }` | Release the session. |
| `mcp__ship__view_preview_snapshot` | `{ session_id }` | PNG + DOM snapshot — useful for screenshots in evals or QA reports. |

### 1.4 Theme + collections

| Tool | Signature | Use |
|---|---|---|
| `mcp__ship__view_theme_tokens` | `{}` | The active theme's design tokens (`--background`, `--foreground`, `--card`, `--ring`, …). Read before generating CSS so you stay token-locked. |
| `mcp__ship__view_collections_schema` | `{}` | Schemas for any project-defined collections (the `<data source="…" />` surface). Use before authoring a `<data>` directive. |
| `mcp__ship__view_annotate` | `{ view, selector, note }` | Pin a TODO/comment to a DOM node — surfaces in the dev panel. |

### 1.5 Skill state

| Tool | Signature | Use |
|---|---|---|
| `mcp__ship__set_skill_var` | `{ stable_id, key, value }` | Persist a per-skill var across runs (e.g. last-used theme, eval-run id). Reads via the resource `ship://skills/<id>`. |

### 1.6 Resources (ship://)

URI templates read via `mcp__ship__ReadMcpResource`. Read-only context.

| URI | What it returns |
|---|---|
| `ship://components` | The aggregated Custom Elements Manifest for every `<sui-*>` primitive (canonical agent-side catalog). |
| `ship://project_info` | Cwd, git status, active workspace. |
| `ship://skills`, `ship://skills/<id>` | Skill catalog + per-skill vars. |
| `ship://workspaces`, `ship://sessions` | Project workspace topology. |
| `ship://jobs`, `ship://jobs/<id>` | Active jobs + their last step. |
| `ship://events`, `ship://events/<n>` | Recent event stream (last `n`). |
| `ship://mesh` | Other agents on the local mesh. |
| `ship://session/<path>` | Files under `.artifacts/<path>` of the active session. |

---

## 2. LSP CLI

For agents that don't have the MCP server attached but do have a shell.
Lives at `packages/sui-lsp/src/cli.ts`.

```sh
# Diagnostics for a single file → JSON array on stdout
bun packages/sui-lsp/src/cli.ts diagnostics views/tasks/page.sui

# Hover info at a cursor
bun packages/sui-lsp/src/cli.ts hover views/tasks/page.sui --line 12 --col 8

# Tag / signal / route deps used by the file
bun packages/sui-lsp/src/cli.ts dependencies views/tasks/page.sui
```

Output shape (diagnostics):

```json
[
  { "line": 12, "col": 4, "severity": "error", "message": "unknown modifier \"enter\"", "source": "sui-compiler" },
  { "line": 18, "col": 9, "severity": "warning", "message": "<data> directive without a `name` attr", "source": "sui-lint" }
]
```

The LSP CLI is the **same analyser** that `mcp__ship__sui_diagnostics`
wraps — use whichever surface is closer to your runtime.

---

## 3. `@ship/sui-eval` — regression scoring

When an agent authors a `.sui` view, `sui-eval` runs it against a spec
and scores the output. Source: `packages/sui-eval/src/index.ts`.

```ts
import { runEval, type EvalTask } from '@ship/sui-eval'

const task: EvalTask = {
  description: 'A counter with +1/-1 buttons that resets to 0.',
  catalog: await loadCatalog(),                    // ship://components dump
  spec: {
    mount: (el) => { /* assertions here */ },
    interactions: [
      { action: 'click', selector: 'button[data-action="bump-up"]' },
      { assert: 'text', selector: 'strong', value: '1' },
    ],
  },
}

const result = await runEval(task, { model: 'sonnet' })
// → { score: 0.92, sub: { compile: 1, scope: 1, behaviour: 0.83 }, report: '…' }
```

Sub-scores:
- **compile** — does it parse + lower without errors?
- **scope** — does every signal / directive / tag resolve?
- **behaviour** — does it pass the spec's interaction script in jsdom?

`sniffEscapeHatches(output)` flags illegal escapes (raw `<script>`,
`@ts-ignore`, `eval`, custom imports outside the importmap).

---

## 4. Skills

Two files own the agent-facing prose:

| File | Audience | Use |
|---|---|---|
| `docs/ship-sui/SKILL.md` | Loaded automatically when the `ship-sui` skill activates | Architecture, hard rules, the four layers. |
| `docs/ship-sui/references/agent-authoring.md` | The do-now checklist | Step-by-step view scaffold, common mistakes, the green-path workflow. |

Activate via `mcp__ship__set_skill_var({ stable_id: 'ship-sui', key: 'active', value: 'true' })` or load the file directly.

---

## 5. Gaps + non-surfaces

What an agent might reasonably expect but does NOT exist:

- **No formatter for `.sui`.** `prettier --plugin sui` is not implemented. Agents emit hand-formatted output.
- **No browser-side LSP.** `@ship/sui-lsp` is Node/CommonJS only. The tutorial editor in `apps/docs/` shows compile diagnostics but cannot offer hover or autocomplete sourced from the LSP.
- **No multi-file scaffold mutation tool.** `scaffold_view` produces the skeleton once; subsequent multi-file edits go through `view_write_source` one file at a time.
- **No live-edit collision detection.** Two agents writing to the same view concurrently get last-writer-wins. Use mesh status (`update_mesh_status`) to coordinate.
- **No assertion-style step gate for `sui-eval`.** The scoring is informational; you have to read the score yourself and decide whether to retry.
