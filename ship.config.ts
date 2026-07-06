// Extension entry for the ship repo's own source space — manifest + TS
// declare-forms replacing every JSONC/MD noun previously under `.ship/`.
// Per format-migration Wave 4 (phase A): rules, collections, triggers,
// MCP and pipelines move to declare-forms here; the four file readers +
// the ConfigWatcher DOMAINS entries for pipelines/rules/collections are
// deleted from the runtime (census-format-migration.md §4 row 3).
//
// File loaded by the Phase-2 resolver from the space root
// (`space_loader::read_ship_ts`); see also decision-space-scan.md for
// the rename from `ship.ts` to `ship.config.ts`.

import {
  defineCollection,
  defineMcpServer,
  definePipeline,
  defineRule,
  defineTrigger,
} from "ship";

// ─── MCP servers ──────────────────────────────────────────────────────────────
// Mirrors `.ship/mcp.jsonc`'s 7 entries. `disabled` is a runtime gate
// (kept on the descriptor; not part of `@ship/stack`'s `McpServerDef` —
// the W3 harvest maps it onto the compiler type via
// `agent_config_eval::declarations::McpServerDeclare::into_mcp_server`).
defineMcpServer({
  id: "antigravity",
  server_type: "stdio",
  command: "antigravity",
  args: ["mcp"],
  env: {},
  url: null,
  scope: "project",
});

defineMcpServer({
  id: "rust-docs",
  server_type: "http",
  command: null,
  args: [],
  env: {},
  url: "http://127.0.0.1:7771/mcp",
  scope: "project",
});

defineMcpServer({
  id: "frontend-docs",
  server_type: "http",
  command: null,
  args: [],
  env: {},
  url: "http://127.0.0.1:7772/mcp",
  scope: "project",
});

defineMcpServer({
  id: "mcp-docs",
  server_type: "http",
  command: null,
  args: [],
  env: {},
  url: "http://127.0.0.1:7773/mcp",
  scope: "project",
});

defineMcpServer({
  id: "helm",
  server_type: "http",
  command: null,
  args: [],
  env: {},
  url: "http://localhost:3000/mcp",
  scope: "project",
});

defineMcpServer({
  id: "github",
  server_type: "stdio",
  command: "npx",
  args: ["-y", "@modelcontextprotocol/server-github"],
  env: { GITHUB_TOKEN: "${GITHUB_TOKEN}" },
  url: null,
  scope: "project",
});

defineMcpServer({
  id: "playwright",
  server_type: "stdio",
  command: "npx",
  args: ["@playwright/mcp@latest"],
  env: {},
  url: null,
  scope: "project",
});

// ─── Rules ───────────────────────────────────────────────────────────────────
// Per `decision-morning-answers.md` §2, rules collapse to the thinnest
// form: id + content string + cascade placement (apply scope). The
// previously-authored `.ship/rules/<id>.md` markdown bodies move inline
// here as TS strings. `apply_scope` is `"always"` for the unconditional
// rules (the prior frontmatter defaults) and a `{ globs: [...] }` object
// for the globs-keyed files where frontmatter `alwaysApply: false`.

defineRule({
  id: "overview",
  content: `# Overview

Ship is a runtime for secure, full-stack, agentic applications. A daemon dispatches
agents, advances pipelines, and event-sources the result. The event log is truth;
disk is a projection. Read this first, then the binding rules below.

Internally, Ship compiles \`.ship/\` config into harness-native agent files — dev
infrastructure for Ship's own agents, not a product surface.

## Scope model — ownership = sync

Every event belongs to one scope, owned by whoever holds the authoritative copy:

* **Space** — collaborative work inside a \`.ship/\` directory (views, agents, skills, pipelines). One daemon serves many spaces; each partitions into its own log. Syncs to collaborators.
* **Account** — identity, library, settings. Syncs across the account's own devices.
* **Runtime** — execution state (sessions, actors, processes, daemon). Machine-local in \`runtime.db\`; snapshot-restorable, never continuously synced.

Substrate (L0) is fat — it holds the logic and the truth. Logic moves down, not up.

## Where to go next

* Binding rules: \`architecture.md\`, \`engineering.md\`, \`agent-discipline.md\`, \`job-protocol.md\`.
* How it fits together: the \`ship-architecture\` doc-skill (\`docs/ship-architecture/\`).
* How we document / agent memory: the \`ship-docs\` and \`memento\` skills.`,
  applyScope: "always",
});

defineRule({
  id: "architecture",
  content: `# Architecture

* Four layers: L0 (substrate), L1 (platform primitives), L2 (built-in apps), L3 (user apps). (Current code map: the \`ship-architecture\` skill — descriptive, not a source of intent.)
* L2 has no privileged access. Built-in apps consume the same SDK as L3 — no \`crate::services::*\` shortcuts in L2 code.
* Event-sourced storage by default. Use the kernel + projections. Other storage requires a registered exception with a written reason.
* One contract per boundary, one source of truth. SDK from one operation descriptor; primitive schemas validated by one validator.
* Agents are actors. Actor lifecycle and supervision are L0; agent profiles plus compilation to provider config are L1. Don't conflate.
* Capability, not mechanism: L1 declares results; L0 owns how. A port/method names a domain need, not an IO verb (\`config.get\`, not \`fs.read\`).
* L1 reaching for a primitive (fs/process/broker) with specific paths = driving mechanism — name the result, move the procedure behind a capability.
* Group capabilities by what callers want, not what things are.
* ConnectRPC is the only surface Ship exposes — a new capability is a Connect op or it doesn't exist; no new REST/CLI/MCP endpoints. (Integrating an external surface, e.g. an MCP server, is a capability; exposing a second one is not.)
* Consumers (UI, CLI, agents) compose capabilities; they never cause a backend surface to exist. A new op is justified by a capability, not a consumer's convenience.
* No ADR convention exists — don't suggest writing one. Architectural decisions that constrain future work live in the Helm blueprint (system/subsystem charters); smaller decisions update a rule file in \`.ship/rules/\`.`,
  applyScope: "always",
});

defineRule({
  id: "architecture-decisions",
  content: `# Locked architectural decisions (do not re-litigate)

> \`CLAUDE.md\` is mode-compiled (\`ship use\` regenerates it), so durable decisions live HERE, not there. This file is the source of truth for the cross-session principles that keep getting re-taught.

## Multi-repo bounded contexts
Split at a **stable contract with an independent consumer**; do NOT split tightly-coupled internals (no contract = friction with no guardrail). Contract boundaries are agent guardrails: an agent in a repo can only touch that repo's contract surface (this is *why* we split — to stop the shipd/axum clobbering).

Target topology:
- **contract** — SDK + protos + types. The linchpin every repo pins.
- **ship** — daemon / backend / services. Implements the contract.
- **sui** — deck + UI stdlib + \`sui-*\` components (the framework). Consumed via the registry.
- **apps** — hub, spaces, site, docs + **\`ship-*\` components** (Ship's app/domain UI). Consume sui + the SDK. (\`ship-*\` do NOT belong in sui.)
- **cloud** — thin RPC service. Own repo when ready (cheap; thin, proto-only seam).
- **ship-ops** — peer consumer of sui; has its own backend.
- Do NOT split: L0↔L1 (internal layering — enforce with lint, not a repo), \`cli\`, \`ship-code\`.

## Components
- \`sui-*\` → sui; \`ship-*\` → apps.
- Consumed via the **registry** (the eventual mechanism), NEVER relative-path workspace-walking. **Rule: no relative paths to pull components from outside the repo.** The \`workspace.json\` /\`/_deck/catalog\` approach is dead — the registry replaces it.

## JS runtime / SSR
- \`ship-js\` (deno_core/V8) is the untrusted-execution core, including deck SSR (deck serves untrusted user/agent code in-process on axum — the isolate is a requirement, not accidental coupling).
- Toolkit-with-contracts, not one-runtime-everywhere: V8 (untrusted core) · bun/Hono (trusted fast SSR, in progress) · render-eval (being deprecated → kept as an optional strict/perf adaptor). JS execution is the core implementation.

## Hub port
- It is a **refactor, not a copy.** Shell is the target UX but built on a prior API with YAGNI. Per surface, two questions: (1) do we need it? (2) is it on the real SDK or a dead API? Drop YAGNI. **Depth matters** — sub-routes that *do* things, not just list views.
- UX is the **design team's** to own (the apps extraction enables their cloud workspace to mirror the repo). Don't guess UX. The shell home's **dock** was a preferred design worth bringing back — but the home design is the team's call.`,
  applyScope: "always",
});

defineRule({
  id: "workflow",
  content: `# Workflow

The binding workflow. Full SSOT (model, surfaces, rationale): helm \`data/wiki/workflow.md\`;
layout decisions: helm \`data/wiki/conventions.md\`. This file is the enforceable subset.

* **One space.** \`~/dev/Ship/\` holds every repo. \`~/dev/ship\` + \`~/dev/sui\` are symlinks into it — never a second clone.
* **One worktree home.** \`~/dev/Ship/worktrees/<repo>/<slug>\` (set via \`ship config worktrees.dir\`). A worktree exists IFF a helm mission has a job for it — the mission dictates worktrees.
* **Branch flow: job → integration → main.** \`job/<slug>\` = one job, one worktree, branched from the mission's integration branch. **integration branch** = ONE long-lived branch per mission (e.g. \`feat/sui-rebuild\`); jobs squash-**LAND** onto it. \`main\` = stable; integration squash-**RELEASES** to main + tag, **operator-gated**. Long-lived feature branches are intended — do NOT release while work is in progress.
* **Mission on the long-lived feature branch.** The mission agent owns the integration branch; subagents work on \`job/*\` branches that the mission agent LANDS in.
* **Land-or-clean gate.** Every branch/worktree is ACTIVE (open job) | LANDED (squash-merged → branch + worktree DELETED) | CLEAN (no unique value → DELETED). Anything else is a leak. **Parked = lost or duplicated work** — never park.
* **It's us.** Operator-driven sessions + \`omp\` as compute for jobs. No "other agents" to tiptoe around — every branch is ours; land it or clean it.
* **Git via ship-git.** The land/release run on the GitService leaf (\`VcsPort.land/release\`), not raw \`git\` shell-out. Dispatch IS the helmd RPC \`ship.ops.v1.AgentsService/Dispatch\` (worktree + compile + harness); \`dispatch-portable.sh\` is the thin CLI client that POSTs it.`,
  applyScope: "always",
});

defineRule({
  id: "agent-discipline",
  content: `# Agent Discipline

No placeholders, no TODOs, no fake data. If it's not finished, remove it.
Don't create something that looks better than it is. The next agent treats your scaffold as architecture.
Working files go in your mission's scratch (helm \`data/missions/<id>/scratch/\`) — \`.artifacts/\` is RETIRED; never create one. \`.ship/\` is project configuration (agents, skills, pipelines, views, permissions) — edit it when the task requires a real config change; don't treat it as scratch.

Always respect \`.gitignore\`. Never \`git add -f\`.

## Craft
- Brevity is a feature. One sentence beats three. Don't narrate your reasoning.
- Surface problems, never pass them. Don't fix out-of-scope issues, but never bury one — note it and move on. A known bug is manageable; an invisible one compounds.
- Use the API, never circumvent it. No raw SQL, no shell scripts bypassing interfaces. If an API exists, use it; if it doesn't, flag it.
- Don't reach for an LLM when a deterministic tool will do. Agentic programs carry more entropy than classical ones — idempotent jobs need deterministic tools.
- Don't shoehorn. Recent conversation, commits, and context are not relevant to every task. Do the work that was asked, not a remix of what you just saw.
- Document the present, not the past. Code, comments, and skills state what is — no "previously," no "we used to," no narrating the alternatives you considered. The next agent never saw your conversation.
- Don't retry tests hoping for different output. Read the failure, fix the root cause, run once more. Two failures from the same cause are a bug to diagnose, not a flake to retry.

## Intent
- Code, tests, and consumers are not evidence of intent. Don't argue "X is used by N things" — only the rules and the operator settle intent.
- **Is not ought.** That a thing exists doesn't mean it should. Before proposing to fix, extend, or option a repair for ANYTHING: date it, trace its LIVE consumers (its own tests and unstable-gated callers don't count as life), and check whether something newer superseded it in place. Evaluate deletion before repair, always. Optioning fixes for a mausoleum is slop with extra steps.
- Don't assume why something is absent — it may be a deliberate cut or an accident. Check \`git blame\` / dates / commit messages; if still unclear, ask. Never restore or re-delete on a guess; in a merge, don't assume one branch added what another removed.
- Ask, then build. "Is this the right approach?" before 500 lines saves a rewrite. When intent is unclear, ask a specific, decidable question — don't resolve ambiguity by assumption.
- Read before you assume. Stale context produces stale output; verify your understanding is current before acting.
- Claim only what you ran. "Tests pass" means you ran them and can show the output. Test the behavior, not the claim — code that "should work" doesn't.`,
  applyScope: "always",
});

defineRule({
  id: "engineering",
  content: `# Engineering

* Test first. Cover behavior changes and meaningful failure paths.
* Test behavior, not implementation: assert observable outputs — emitted events, op responses, returned results — not private state, internal call counts, or exact log wording. A behavior-preserving refactor must not break a test.
* A test fossilizes whatever it asserts. Don't write one on guessed behavior — confirm intent first. Brittle, implementation-coupled tests are deletion candidates.
* Editing \`.sui\` / \`*.controller.ts\`? Read the \`ship-sui\` skill first (SUI is not in training data) — lint then compile per edit.
* File caps: soft 300 (split before exceeding), hard 400. \`// FILE_CAP_OVERRIDE: <reason>\` only when genuinely warranted.
* One way to do things. If a solution exists, use it. No parallel systems.
* Rust logic in runtime. CLI is thin transport. Web fetches from the daemon only.
* \`just build\` for dev. Stage explicit files. Commit types: feat, fix, refactor, test, docs, chore.
* No \`Co-Authored-By:\` lines. No model/agent attribution in commits.
* Id generation is a real split, not drift: \`ship_substrate::gen_ulid()\` for sortable, monotonic ids on anything ordered by creation time (events, sessions, artifacts, pipeline runs); \`ship_substrate::gen_nanoid()\` for short human-facing ids (space/project ids), which delegates to the compiler's single 56-char ambiguous-char-free alphabet. A crate that already depends on \`ship-substrate\` calls these helpers, not \`ulid::Ulid::new()\` / \`nanoid::nanoid!()\` directly. Raw \`nanoid::nanoid!()\` stays legitimate for ephemeral, non-human-facing internal names (sandbox/socket/process ids) where the length is call-site-specific and no shared helper applies; a leaf crate that can't depend on \`ship-substrate\` (layering) reimplementing the monotonic-ulid pattern locally is a layering violation — file the coupling.`,
  applyScope: "always",
});

defineRule({
  id: "pre-launch",
  content: `# Pre-launch (delete this file at launch)

No users and no persisted data to protect yet. Until launch:

- No migration code, compat shims, or deprecation paths. Change the thing directly.
- Breaking old behavior in a refactor is expected, not a regression. A test that asserts removed behavior is deleted, not satisfied — the bar is green on the new design, not green on the old.
- Replace, don't layer. No parallel old/new paths kept "for safety." Delete the old.`,
  applyScope: "always",
});

// ─── Pipelines ───────────────────────────────────────────────────────────────
// Mirrors `.ship/pipelines/<name>.jsonc`. The pipeline runner's DAG and
// gate shapes live in `ship_substrate::events::pipeline::*` (the L0
// pipeline events) and `ship_pipelines::parse::normalize_pipeline` —
// the TS shapes ship-contracts lands in Wave 1 are the declared-surface
// projection; the orchestrator dispatches them via the descriptor.

definePipeline({
  name: "smoke",
  steps: [
    {
      id: "smoke-step-1",
      step_specs: [{ id: "smoke-step-1", prompt: "echo \"smoke step-1: scratch work\" > smoke.txt\ngit add smoke.txt\ngit -c user.name=smoke -c user.email=smoke@ship.dev commit -m \"smoke: step-1 creates smoke.txt\"\n" }],
    },
    {
      id: "smoke-step-2",
      step_specs: [{ id: "smoke-step-2", prompt: "echo \"smoke step-2: modify\" >> smoke.txt\ngit -c user.name=smoke -c user.email=smoke@ship.dev commit -am \"smoke: step-2 appends to smoke.txt\"\n" }],
    },
    {
      id: "smoke-step-3",
      step_specs: [{ id: "smoke-step-3", prompt: "echo \"smoke step-3: verify\" >&2\ntest -f smoke.txt && grep -q \"step-2\" smoke.txt\n" }],
    },
  ],
  auto_merge: false,
});

definePipeline({
  name: "shell-smoke",
  steps: [
    {
      id: "one",
      step_specs: [{ id: "one", prompt: "echo \"step 1\"\necho \"step 1\" > smoke.txt\ngit add smoke.txt\ngit -c user.name=smoke -c user.email=smoke@ship.dev commit -m \"smoke step 1\"\n" }],
    },
    {
      id: "two",
      after: ["one"],
      step_specs: [{ id: "two", prompt: "echo \"step 2 appended\" >> smoke.txt\ngit -c user.name=smoke -c user.email=smoke@ship.dev commit -am \"smoke step 2\"\n" }],
    },
    {
      id: "three",
      after: ["two"],
      step_specs: [{ id: "three", prompt: "test -f smoke.txt && grep -q \"step 2 appended\" smoke.txt\n" }],
    },
  ],
  auto_merge: false,
});

definePipeline({
  name: "multi-harness-smoke",
  steps: [
    {
      id: "implement",
      step_specs: [{ id: "implement", agent: "commander", harness: "claude" }],
    },
    {
      id: "verify",
      after: ["implement"],
      step_specs: [{ id: "verify", prompt: "cargo check --workspace 2>&1 | tail -5\n" }],
    },
    {
      id: "review",
      after: ["verify"],
      step_specs: [{ id: "review", agent: "gatekeeper", harness: "claude" }],
      gate: {
        require_changed: [{ match: ["**/*.rs"], min: 1 }],
        forbid_changed: [{ match: [".artifacts/**"], min: 0 }],
      },
    },
  ],
  auto_merge: true,
});

definePipeline({
  name: "multi-provider-smoke",
  steps: [
    {
      id: "implement",
      step_specs: [{ id: "implement", agent: "commander", harness: "claude" }],
    },
    {
      id: "verify",
      after: ["implement"],
      step_specs: [{ id: "verify", prompt: "cargo check --workspace 2>&1 | tail -5\n" }],
    },
    {
      id: "review",
      after: ["verify"],
      step_specs: [{ id: "review", agent: "gatekeeper", harness: "claude" }],
      gate: {
        require_changed: [{ match: ["**/*.rs"], min: 1 }],
        forbid_changed: [{ match: [".artifacts/**"], min: 0 }],
      },
    },
  ],
  auto_merge: true,
});

definePipeline({
  name: "ship-dev",
  steps: [
    {
      id: "implement",
      step_specs: [{ id: "implement", agent: "rust-lane", harness: "claude" }],
    },
    {
      id: "verify",
      after: ["implement"],
      step_specs: [{ id: "verify", prompt: "cargo check --workspace 2>&1 | tail -5\ncargo test --workspace --no-fail-fast 2>&1 | tail -10\n" }],
    },
    {
      id: "review",
      after: ["verify"],
      step_specs: [{ id: "review", agent: "gatekeeper" }],
      gate: {
        require_changed: [{ match: ["**/*.rs"], min: 1 }],
        forbid_changed: [{ match: [".artifacts/**"], min: 0 }],
      },
    },
  ],
  auto_merge: true,
});

definePipeline({
  name: "view-session-e2e",
  steps: [
    {
      id: "open",
      step_specs: [{ id: "open", prompt: "set -euo pipefail\n: \"${SHIP_DAEMON_TOKEN:?SHIP_DAEMON_TOKEN must be set}\"\nurl=\"${SHIP_DAEMON_URL:-http://127.0.0.1:9315}\"\n: \"${SHIP_MISSION_SCRATCH:=$(pwd)}\"\n\nresp=$(curl -fsS -X POST \"$url/api/view-sessions\" \\\n  -H \"Authorization: Bearer $SHIP_DAEMON_TOKEN\" \\\n  -H \"x-ship-agent-id: pipeline.e2e\" \\\n  -H \"content-type: application/json\" \\\n  -d '{\"template\":\"canvas/blank\",\"title\":\"e2e\"}')\necho \"$resp\" | tee \"$SHIP_MISSION_SCRATCH/view-session-open.json\"\nview_id=$(echo \"$resp\" | sed -n 's/.*\"view_id\":\"\\([^\"]*\\)\".*/\\1/p')\necho \"$view_id\" > \"$SHIP_MISSION_SCRATCH/view-session-id\"\n" }],
    },
    {
      id: "subscribe-and-patch",
      step_specs: [{ id: "subscribe-and-patch", prompt: "set -euo pipefail\n: \"${SHIP_DAEMON_TOKEN:?SHIP_DAEMON_TOKEN must be set}\"\nurl=\"${SHIP_DAEMON_URL:-http://127.0.0.1:9315}\"\n: \"${SHIP_MISSION_SCRATCH:=$(pwd)}\"\nview_id=$(cat \"$SHIP_MISSION_SCRATCH/view-session-id\")\n\n# Subscribe in the background, capture SSE body, look for the beacon.\n( timeout 5 curl -sS -N \"$url/api/view-sessions/$view_id\" \\\n    -H \"Authorization: Bearer $SHIP_DAEMON_TOKEN\" \\\n    -H \"x-ship-agent-id: pipeline.e2e\" \\\n    > \"$SHIP_MISSION_SCRATCH/view-session-sse.log\" ) &\nsub_pid=$!\nsleep 1\n\ncurl -fsS -X POST \"$url/api/view-sessions/$view_id/patch\" \\\n  -H \"Authorization: Bearer $SHIP_DAEMON_TOKEN\" \\\n  -H \"x-ship-agent-id: pipeline.e2e\" \\\n  -H \"content-type: application/json\" \\\n  -d '{\"op\":\"append\",\"node\":{\"kind\":\"text\",\"text\":\"hello\"}}'\n\nwait \"$sub_pid\" || true\ngrep -q 'event: beacon' \"$SHIP_MISSION_SCRATCH/view-session-sse.log\" \\\n  || { echo \"no beacon in SSE log\"; cat \"$SHIP_MISSION_SCRATCH/view-session-sse.log\"; exit 1; }\n" }],
    },
    {
      id: "close",
      step_specs: [{ id: "close", prompt: "set -euo pipefail\n: \"${SHIP_DAEMON_TOKEN:?SHIP_DAEMON_TOKEN must be set}\"\nurl=\"${SHIP_DAEMON_URL:-http://127.0.0.1:9315}\"\n: \"${SHIP_MISSION_SCRATCH:=$(pwd)}\"\nview_id=$(cat \"$SHIP_MISSION_SCRATCH/view-session-id\")\n\ncurl -fsS -X DELETE \"$url/api/view-sessions/$view_id\" \\\n  -H \"Authorization: Bearer $SHIP_DAEMON_TOKEN\" \\\n  -H \"x-ship-agent-id: pipeline.e2e\"\n\n# Owner-validation smoke: a different agent id must get 403 on close.\nstatus=$(curl -s -o /dev/null -w '%{http_code}' \\\n  -X DELETE \"$url/api/view-sessions/$view_id\" \\\n  -H \"Authorization: Bearer $SHIP_DAEMON_TOKEN\" \\\n  -H \"x-ship-agent-id: someone.else\")\ntest \"$status\" = \"404\" \\\n  || { echo \"close: expected 404 after prior close, got $status\"; exit 1; }\n" }],
    },
  ],
  auto_merge: false,
});

// ─── Triggers ────────────────────────────────────────────────────────────────
// Mirrors `.ship/triggers/log-failed-jobs.jsonc`. Single trigger definition.

defineTrigger({
  id: "log-failed-jobs",
  trigger: { event: "pipeline.run.failed" },
  actions: [
    {
      type: "event.emit",
      event_type: "trigger.job_failed_notice",
      payload: {
        job_id: "{{ event.entity_id }}",
        triggered_by: "{{ event.id }}",
      },
    },
  ],
});

// ─── Collections ─────────────────────────────────────────────────────────────
// Mirrors `.ship/collections/*.jsonc`. Each call here hydrates one
// row into the runtime `SchemaRegistry` at shipd/helmd boot, replacing
// the file-overlay hydration `collection_schemas::hydrate_registry`
// performed previously (deletion map row 3).

defineCollection({
  id: "adrs",
  name: "Architecture Decisions",
  fields: [
    { name: "title", type: "string", required: true, filterable: true },
    {
      name: "status",
      type: "enum",
      values: ["proposed", "accepted", "superseded", "deprecated"],
      filterable: true,
      default: "proposed",
    },
    { name: "context", type: "markdown" },
    { name: "decision", type: "markdown" },
    { name: "consequences", type: "markdown" },
  ],
});

defineCollection({
  id: "architecture-observations",
  name: "Architecture Observations",
  fields: [
    { name: "id", type: "string", required: true, filterable: true },
    { name: "module", type: "string", required: true, filterable: true },
    {
      name: "layer",
      type: "enum",
      values: ["L0", "L1", "L2", "shared", "apps"],
      filterable: true,
    },
    { name: "tag", type: "string", required: true, filterable: true },
    {
      name: "tag_status",
      type: "enum",
      values: ["candidate", "confirmed", "promoted"],
      filterable: true,
    },
    {
      name: "severity",
      type: "enum",
      values: ["info", "low", "medium", "high", "critical"],
      filterable: true,
      sortable: true,
    },
    { name: "citation_file", type: "string" },
    { name: "citation_line", type: "string" },
    { name: "description", type: "markdown", required: true },
    {
      name: "status",
      type: "enum",
      values: [
        "pending",
        "under-review",
        "signed-off",
        "superseded",
        "closed",
      ],
      filterable: true,
      default: "pending",
    },
    { name: "audit_doc", type: "string" },
    { name: "task_id", type: "string" },
    { name: "pass_date", type: "string" },
    { name: "notes", type: "markdown" },
  ],
});

defineCollection({
  id: "claim",
  name: "Claims",
  fields: [
    {
      name: "kind",
      type: "enum",
      values: ["bug", "request", "observation", "hitl"],
      required: true,
      filterable: true,
    },
    {
      name: "by",
      type: "enum",
      values: ["user", "agent", "external"],
      required: true,
      filterable: true,
    },
    {
      name: "target",
      type: "string",
      required: true,
      filterable: true,
    },
    {
      name: "version",
      type: "string",
      required: true,
      filterable: true,
    },
    { name: "body", type: "string", required: true },
    {
      name: "status",
      type: "enum",
      values: ["open", "reconciled", "archived"],
      required: true,
      filterable: true,
      default: "open",
    },
    {
      name: "claimed_at",
      type: "number",
      required: true,
      filterable: true,
      sortable: true,
    },
    { name: "archived_at", type: "string" },
    { name: "reconciliation", type: "object" },
  ],
});

defineCollection({
  id: "ops-gap",
  name: "Ops Catalog Gap",
  fields: [
    {
      name: "service",
      type: "string",
      required: true,
      filterable: true,
    },
    {
      name: "op_name",
      type: "string",
      required: true,
      filterable: true,
    },
    {
      name: "substrate_fn",
      type: "string",
      required: true,
    },
    {
      name: "audience",
      type: "enum",
      values: ["operator", "agent", "both"],
      filterable: true,
    },
    {
      name: "capability",
      type: "enum",
      values: ["C0", "C1", "C2", "C2_ELEVATED", "C3"],
      filterable: true,
    },
    {
      name: "priority",
      type: "enum",
      values: ["P0", "P1", "P2", "deferred"],
      filterable: true,
      sortable: true,
    },
    {
      name: "status",
      type: "enum",
      values: [
        "missing",
        "scaffolded",
        "implemented",
        "rejected-no-wire",
        "in-flight",
      ],
      filterable: true,
      default: "missing",
    },
    { name: "rationale", type: "markdown" },
    { name: "audit_source", type: "string" },
    { name: "consumer_demand", type: "markdown" },
    { name: "dependencies", type: "string" },
  ],
});

defineCollection({
  id: "sui-eval-runs",
  name: "SUI Eval Runs",
  fields: [
    { name: "run_id", type: "string", required: true, filterable: true },
    { name: "lane_id", type: "string", required: true, filterable: true },
    { name: "model", type: "string", filterable: true },
    { name: "harness", type: "string", filterable: true },
    { name: "composite", type: "number", filterable: true },
    { name: "compile_passed", type: "boolean", filterable: true },
    { name: "lint_errors", type: "number" },
    { name: "lint_warnings", type: "number" },
    { name: "worktree", type: "string" },
    { name: "view_files", type: "object" },
    { name: "css_violations", type: "object" },
    { name: "missing_fragments", type: "object" },
    { name: "behavior_passed", type: "number" },
    { name: "behavior_total", type: "number" },
    { name: "behavior_score", type: "number", filterable: true },
  ],
});

defineCollection({
  id: "systems",
  name: "Ship Systems",
  fields: [
    { name: "name", type: "string", required: true, filterable: true },
    {
      name: "layer",
      type: "enum",
      values: ["primitive", "runtime", "interface", "meta"],
      filterable: true,
    },
    {
      name: "status",
      type: "enum",
      values: ["spec", "building", "shipping", "complete"],
      filterable: true,
      default: "spec",
    },
    { name: "summary", type: "string" },
    { name: "current_state", type: "markdown" },
    { name: "desired_state", type: "markdown" },
    { name: "gaps", type: "array" },
    { name: "diagram", type: "markdown" },
    { name: "doc_skill", type: "string", filterable: true },
    { name: "code_paths", type: "array" },
    { name: "depends_on", type: "array", filterable: true },
    { name: "position_x", type: "number" },
    { name: "position_y", type: "number" },
  ],
});
