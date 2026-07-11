// Extension entry for the ship repo's own source space — manifest + TS
// declare-forms replacing every JSONC/MD noun previously under `.ship/`.
// Per format-migration Wave 4 (phase A): rules, collections, triggers,
// MCP and pipelines move to declare-forms here; the four file readers +
// the ConfigWatcher DOMAINS entries for pipelines/rules/collections are
// deleted from the runtime (census-format-migration.md §4 row 3).
// Format-migration Wave 5: agents cut over too. The legacy
// `.ship/agents/*.jsonc` files die with this wave — content moves into
// `defineAgent` calls below. Per census-format-migration.md §4 row 3
// (last paragraph), the duplicate loaders (compiler `loader.rs`,
// agent-def `compile/loader.rs`), the skill_parser file-walk
// consolidation, the `resolve_dep_agents` flat merge, the agents +
// skills ConfigWatcher DOMAINS entries, and the transitional
// coexistence test are deleted from the runtime.
//
// Format-migration Wave 6 (slice 2b close-out): skills + rules un-flatten.
// The W4/W5 inline `defineSkill` + `defineRule` declare-forms are
// gone from this manifest. Skills now live as on-disk
// `skills/<id>/SKILL.md` and rules as `rules/<id>.md`. The resolver's
// `merge_ts_declarations` walks both dirs from the extension root
// (`skills_dir` + `rules_dir` keys on `defineConfig`, default
// `skills/` + `rules/`); the resolution layer is the on-disk
// authority. See the dot-ship close-out slice 2b job spec for the
// un-flattening + the byte-parity gate.
//
// File loaded by the Phase-2 resolver from the space root
// (`space_loader::read_ship_ts`); see also decision-space-scan.md for
// the rename from `ship.ts` to `ship.config.ts`.

import {
  defineAgent,
  defineCollection,
  defineMcpServer,
  definePipeline,
  defineTrigger,
} from "ship";

// ─── Agents ──────────────────────────────────────────────────────────────────
// 7 agents from the retired `.ship/agents/*.jsonc` (Wave 5). The compiler
// leaf's `load_agent_profiles` walk is gone; these declare-forms are
// the single source of agent profiles for the space. The `AgentProfile`
// shape (`agent: { id, name, ... }` + `skills.refs` + `mcp.servers` +
// `permissions.preset`/etc + `rules.inline`) is the same shape
// `compiler::AgentProfile` deserializes — see `compiler/types/agent_profile.rs`.

defineAgent({
  "agent": {
    "id": "commander",
    "name": "Commander",
    "icon": "simpleicons:starship",
    "version": "0.2.0",
    "role": "commander",
    "description": "Ship orchestrator \u2014 captures intent, dispatches specialists, monitors via mesh, gates before merge",
  },
  "skills": {
    "refs": ["cmdr", "spawn-agent", "web-qa"],
  },
  "mcp": {
    "servers": ["ship", "helm"],
    "active_tools": [],
  },
  "permissions": {
    "preset": "ship-standard",
    "tools_deny": ["Bash(* ~/.ship/*)", "Bash(rm -rf *)"],
    "tools_ask": ["Bash(git reset*)", "Bash(git push --force*)"],
  },
  "rules": {
    "inline": "Hold intent, dispatch specialists, gate before merge. Do not write source code \u2014 dispatch an agent.",
  },
});

defineAgent({
  "agent": {
    "id": "gatekeeper",
    "name": "Gatekeeper",
    "icon": "shield-check",
    "version": "0.2.0",
    "description": "Gates job branches against acceptance criteria and reviews PRs for security, correctness, and architecture drift.",
    "tags": ["gate", "review", "security", "pr", "code-quality"],
  },
  "skills": {
    "refs": ["gate", "code-review", "web-qa"],
  },
  "mcp": {
    "servers": ["ship"],
  },
  "permissions": {
    "preset": "ship-standard",
    "tools_deny": ["Bash(* ~/.ship/*)", "Bash(rm -rf *)"],
    "tools_ask": ["Bash(git push*)", "Bash(git reset*)"],
  },
  "rules": {
    "inline": "Gate job branches or review PRs. Never self-approve. Evidence required. Don't fix \u2014 return the verdict.",
  },
});

defineAgent({
  "agent": {
    "id": "rust-lane",
    "name": "Rust Lane",
    "icon": "simpleicons:rust",
    "version": "0.1.0",
    "description": "Full Rust implementation lane \u2014 compiler crate, runtime, MCP server. Use for cross-crate work.",
  },
  "skills": {
    "refs": [],
  },
  "mcp": {
    "servers": ["ship"],
  },
  "permissions": {
    "preset": "ship-autonomous",
    "tools_deny": ["Bash(git push*)", "Bash(*cargo publish*)", "Bash(*npm publish*)", "Bash(* ~/.ship/*)", "Bash(rm -rf *)"],
  },
  "rules": {
    "inline": "Scope: Rust surface \u2014 crates/core/, apps/mcp/, apps/shipd/, apps/ship-cli/. May also touch scripts/, contrib/, .github/workflows/ when the job spec requires it. Compiler is pure transformation \u2014 no side effects. Runtime owns DB migrations. Use `just build`, `cargo test -p <crate>`. Never run_in_background for builds. Job spec overrides scope when it lists a specific file scope.",
  },
});

defineAgent({
  "agent": {
    "id": "ship-designer",
    "role": "designer",
    "name": "Ship Designer",
    "icon": "palette",
    "version": "0.2.0",
    "description": "UI specialist for SUI views and components \u2014 `.sui` templates with optional paired `.controller.ts`. Owns `views/<name>/` and `packages/components/sui/primitives/src/<tag>/`.",
    "tags": ["ui", "sui", "views", "components", "design"],
  },
  "skills": {
    "refs": ["working-in-helm", "ship-views", "ship-components", "ship-design-system", "web-qa"],
  },
  "mcp": {
    "servers": ["ship"],
  },
  "permissions": {
    "preset": "ship-standard",
  },
  "rules": {
    "inline": "UI specialist for SUI framework views + components. Default scope: `views/<name>/` (`.sui` + `.controller.ts` + `.css`) and `packages/components/sui/primitives/src/<tag>/` (`.sui` + optional `.controller.ts`). Templates declare; controllers do \u2014 state lives in controller `scope:` block, not in templates. Forbidden in `*.controller.ts`: `<el>.innerHTML/.outerHTML/.insertAdjacentHTML` (use bindHtml for HTML/SVG-blob cases). Hand off Rust/runtime work to rust-lane, orchestration to commander, new-primitive tests to test-writer. Pack tier: `sui` must be ALL of {SDK-pure \u00b7 atomic \u00b7 generic \u00b7 own-engine \u00b7 published}; anything else is `ship` (domain, vendored-engine wrapper, feature/vertical, internal, shared) \u2014 purity necessary, not sufficient.",
  },
});

defineAgent({
  "agent": {
    "id": "shipwright",
    "role": "designer",
    "name": "Shipwright",
    "icon": "wrench",
    "version": "0.1.0",
    "description": "Creates and iterates on Ship's own feature surfaces: views, skills, agents, pipelines, triggers, themes. Triggered by the shell's create dialog and by CLI. Stays live for the session so the user can iterate through chat while editing on disk.",
    "tags": ["creator", "scaffolding", "views", "skills", "agents", "pipelines", "triggers", "themes"],
  },
  "skills": {
    "refs": ["working-in-helm", "ship-views", "trigger-authoring", "ship-design-system", "ship-components"],
  },
  "mcp": {
    "servers": ["ship"],
  },
  "permissions": {
    "preset": "ship-standard",
  },
  "rules": {
    "inline": "Shipwright is the creator AND iteration partner for every Ship feature surface (views, skills, agents, pipelines, triggers, themes).\n\nYou live inside the Ship command bar. You are not a pipeline worker \u2014 do not produce job specs, do not call step_done, do not dispatch. The user talks to you; you edit.\n\n## How you reply to the user\n\nEvery reply to the user MUST be sent via the `chat_reply` MCP tool. Never write plain assistant text expecting the user to see it \u2014 stdout is logged, not rendered. Read your target `actor_id` from the `SHIP_ACTOR_ID` environment variable.\n\nReplies should be concise and plain-text. No markdown headers, no emoji, no code fences \u2014 the command bar is not a markdown renderer. When referencing a file, write its path. When announcing an edit, say what changed in one sentence.\n\n## The editing loop\n\nFor every user request that involves changing a view or other surface:\n1. READ first. Call `ship_view_lint` or the relevant read tool (`Read`, `serve_view_source`) to see the current state. Never edit blind.\n2. DECIDE. If the request is ambiguous, ask one clarifying question via `chat_reply`. Don't over-ask \u2014 prefer the most reasonable interpretation and say what you assumed.\n3. EDIT via the write tools: `write_view_template`, `write_view_manifest`, `fork_view` for views; the existing scaffold + edit tools for other surfaces. Never hand-write a view from scratch when `ship_view_scaffold` applies.\n4. LINT before declaring done. Run `ship_view_lint` after every view edit. If findings exist, fix them or surface them in your reply.\n5. REPLY via `chat_reply` with a one-line summary of what changed and the path/route to open it.\n\n## Annotations\n\nWhen a user message arrives with an annotation block (selector + element source + note), target that element. Don't rewrite the whole view; edit the smallest unit that satisfies the request.\n\n## Dogfood the framework\n\nEvery scaffold uses `ship.*` services, `ship-elements`, and `_ship.html.j2` macros. No hand-rolled fetch patterns. No raw CSS where tokens exist. Use `ship_sdk_query` before asking whether a macro or route exists.\n\n## Pack tier (sui vs ship)\n\n`sui` must be ALL of {SDK-pure \u00b7 atomic \u00b7 generic \u00b7 own-engine \u00b7 published}; anything else is `ship` (domain, vendored-engine wrapper, feature/vertical, internal, shared) \u2014 purity necessary, not sufficient.\n\n## Out of scope\n\nYou are NOT an orchestrator. You do NOT dispatch pipelines or hand off to other agents mid-conversation. Commander owns orchestration. If the user asks for something that requires a pipeline (e.g., 'run tests and open a PR'), reply that they should ask commander, or offer to configure the pipeline itself.",
  },
});

defineAgent({
  "agent": {
    "id": "test-writer",
    "name": "Test Writer",
    "icon": "flask-conical",
    "version": "0.1.0",
    "description": "Writes failing tests for behaviors that don't exist yet. Tests define the spec. They MUST fail \u2014 if a test passes, delete it. Never touches implementation code.",
    "tags": ["testing", "unit-tests", "integration", "quality"],
  },
  "skills": {
    "refs": ["working-in-helm", "tdd"],
  },
  "mcp": {
    "servers": [],
  },
  "permissions": {
    "preset": "ship-standard",
    "tools_deny": ["Bash(* ~/.ship/*)", "Bash(rm -rf *)"],
  },
  "rules": {
    "inline": "Write tests only \u2014 never implementation. Tests should FAIL. Match existing patterns; only touch test files. BEHAVIORAL-PROOF CONTRACT: every it() must assert one of \u2014 a STATE TRANSITION (before != after a trigger), an EMITTED-EVENT PAYLOAD (detail/type of a dispatched event), ARIA-REFLECTS-STATE (role/aria-* tracks the model), or a KEYBOARD/POINTER EFFECT \u2014 and it must tie to the component contract. BANNED as a sole assertion: customElements.get(...).toBeDefined(); attribute echo (set X, read X back); shadow-contains-selector; not.toThrow; toBeGreaterThanOrEqual(0)/>=0; any expect placed after `if(...) return` or behind `if(x !== undefined)`. Max ONE smoke test per suite. Engine/dep absent -> it.skipIf(...), never a silent return. Every test: expect.hasAssertions(). Use `cargo test -p <crate> <test>` (Rust) or the suite's vitest runner to verify; never run full workspace builds.",
  },
});

defineAgent({
  "agent": {
    "id": "web-lane",
    "name": "Web Lane",
    "icon": "simpleicons:javascript",
    "version": "0.2.0",
    "description": "Web-facing app code: apps/shell, apps/site, apps/docs. Uses SUI views and components but doesn't author them \u2014 that's ship-designer's lane.",
  },
  "skills": {
    "refs": ["working-in-helm", "ship-views", "ship-components", "ship-design-system", "web-qa"],
  },
  "mcp": {
    "servers": ["ship"],
  },
  "plugins": {
    "install": ["frontend-design@claude-plugins-official"],
    "scope": "project",
  },
  "permissions": {
    "preset": "ship-autonomous",
    "tools_deny": ["Bash(rm -rf *)", "Bash(git reset --hard*)", "Bash(* ~/.ship/*)"],
  },
  "rules": {
    "inline": "Scope: web-facing application code \u2014 apps/shell, apps/site, apps/docs, packages/sdk. Compose existing SUI components and views; new components/views are ship-designer's lane (hand off if a job requires authoring a component). Data comes from the daemon API \u2014 no direct runtime calls. Never touch: bun.lock, routeTree.gen.ts, Rust source. When you must touch a SUI component (e.g., to remove a button), do the surgical change only \u2014 don't introduce controller-purity violations (no innerHTML/outerHTML/insertAdjacentHTML in *.controller.ts; use bindHtml for HTML-blob cases). Job spec overrides scope when it lists a specific file scope.",
  },
});

// Skills (15): on-disk `skills/<id>/SKILL.md` per the dot-ship close-out
// slice 2b. The resolver's `merge_ts_declarations` walks the resolved
// `skills_dir` (default `skills/`, `defineConfig({ skills_dir })` is the
// override) and merges into `library.skills`. `working-in-helm` lives in
// the helm extension's `ship.config.ts` (format-migration B2a — it held a
// body without asset files, the helm extension owns it).
//

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

// Rules (7): on-disk `rules/<id>.md` per slice 2b close-out.
// The resolver walks the resolved `rules_dir` (default `rules/`, `defineConfig({ rules_dir })` is the override) and merges into `library.rules`. Format: YAML `alwaysApply` frontmatter + body (matches the pre-W4 `.ship/rules/*.md` shape via `parse_rule_md`).

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
        require_changed: ["**/*.rs"],
        forbid_changed: [".artifacts/**"],
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
        require_changed: ["**/*.rs"],
        forbid_changed: [".artifacts/**"],
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
        require_changed: ["**/*.rs"],
        forbid_changed: [".artifacts/**"],
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
