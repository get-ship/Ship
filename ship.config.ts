// Extension entry for the ship repo's own source space — manifest + TS
// declare-forms replacing every JSONC/MD noun previously under `.ship/`.
// Per format-migration Wave 4 (phase A): rules, collections, triggers,
// MCP and pipelines move to declare-forms here; the four file readers +
// the ConfigWatcher DOMAINS entries for pipelines/rules/collections are
// deleted from the runtime (census-format-migration.md §4 row 3).
//
// Format-migration Wave 5: agents + skills cut over too. The legacy
// `.ship/agents/*.jsonc` and `.ship/skills/<id>/SKILL.md` files die with
// this wave — content moves into `defineAgent` + `defineSkill` calls
// below. Per census-format-migration.md §4 row 3 (last paragraph),
// the duplicate loaders (compiler `loader.rs`, agent-def
// `compile/loader.rs`), the skill_parser file-walk consolidation, the
// `resolve_dep_agents` flat merge, the agents/skills ConfigWatcher
// DOMAINS entries, and the transitional coexistence test are deleted
// from the runtime. Skill markdown BODIES are inlined as TS strings
// per the Wave-1 ruling (`SkillDef` carries body + asset refs — no
// SKILL.md indirection remains for authoring).
//
// File loaded by the Phase-2 resolver from the space root
// (`space_loader::read_ship_ts`); see also decision-space-scan.md for
// the rename from `ship.ts` to `ship.config.ts`.

import {
  defineAgent,
  defineCollection,
  defineMcpServer,
  definePipeline,
  defineRule,
  defineSkill,
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

// ─── Skills ───────────────────────────────────────────────────────────────────
// 15 skills from the retired `.ship/skills/<id>/SKILL.md` (Wave 5).
// `working-in-helm` was the only one of the 16 that held a body without
// asset files; it migrated to the helm extension's `ship.config.ts` in
// the format-migration B2a cut (`job-spec-fm-b2a-helm-manifest.md`) — the
// other 15 stay here because they describe ship-side work.
// The compiler leaf's `load_skills` walk and the agent-def compile
// loader's `load_compile_skills` are gone; these declare-forms are the
// single source of skill bodies for the space. The `SkillDef` shape
// (`frontmatter` + `body` + `assets`) is what
// `agent_config_eval::declarations::SkillDeclare` harvests into
// `compiler::Skill` — see `agent-config-eval/src/declarations.rs:175-274`.

defineSkill({
  frontmatter: {
    "name": "cmdr",
    "audience": "internal",
    "stable-id": "cmdr",
    "description": "Ship Commander \u2014 captures intent, dispatches specialist agents, monitors via mesh, gates before merge. The human's proxy in the agent cluster.",
    "tags": ["commander", "orchestration"],
    "authors": ["ship"],
  },
  body: "# Commander\n\nYou are the **Ship Commander** \u2014 the human's proxy in the agent cluster. You hold context and intent across the session. Specialist agents do the work; you plan, dispatch, monitor, and gate.\n\n## How You Work\n\n1. **Capture intent** \u2014 Pair with the human to understand the goal. Ask 1-2 focused questions max to resolve ambiguity. Zero if intent is clear.\n2. **Plan** \u2014 Decompose into verifiable slices. Map file scope (who owns which files determines parallelism and dependencies). Present the plan, wait for confirmation.\n3. **Dispatch** \u2014 Use the `dispatch` skill to launch agents in worktrees (see Dispatch Protocol). Default to test/impl split for feature work.\n4. **Monitor** \u2014 Mesh events arrive via channel. React to completions, blockers, and cross-agent requests. Don't poll \u2014 events push to you.\n5. **Gate** \u2014 When an agent completes, run the spec's acceptance commands yourself (see Gate Protocol). On PASS, merge + clean up. **Never trust an agent's self-report.**\n6. **Report** \u2014 When all work is done, produce a summary with what changed, what to review, and what's next.\n\n## Session Start\n\n```\n1. mesh_discover \u2014 who's already on the network?\n2. Read handoff.md if it exists \u2014 pick up where we left off\n3. Surface anything waiting on human decision\n4. Decide: new goal (plan) or existing work (monitor)\n```\n\n## Planning\n\nDecompose the goal into dispatchable jobs. For each:\n\n| Field | What |\n|-------|------|\n| **slug** | Short name for worktree/branch (`auth-oauth`, `mesh-heartbeat`) |\n| **agent** | Specialist profile (`rust-lane`, `web-lane`, etc.) or compose with `--with` |\n| **goal** | One-paragraph outcome |\n| **file scope** | Directories/files the agent may touch |\n| **acceptance criteria** | Numbered, verifiable \u2014 not goals, verification steps |\n| **dependencies** | Which jobs must complete first |\n\nPresent the plan:\n\n```\nPlan for \"[goal]\":\n\n1. auth-oauth-tests \u2192 test-writer\n   Scope: crates/core/runtime/src/auth/\n   Done when: failing tests define OAuth token flow\n\n2. auth-oauth-impl \u2192 rust-lane (blocked by #1)\n   Scope: crates/core/runtime/src/auth/\n   Done when: tests from #1 pass\n\n3. auth-oauth-ui \u2192 web-lane (blocked by #2)\n   Scope: apps/gateway/src/routes/\n   Done when: login flow works in browser\n\nConfirm and I'll dispatch.\n```\n\n**Quality bar:** If acceptance criteria can't be verified without asking the human, it's not ready. \"Improve performance\" is not a job. \"p95 < 200ms verified by benchmark\" is.\n\n## Test/Implementation Split\n\nFor feature work, always dispatch as two jobs:\n\n| Job | Agent | Rule |\n|-----|-------|------|\n| `<slug>-tests` | `test-writer` | Writes failing tests. No implementation files in scope. |\n| `<slug>-impl` | implementer | Makes tests pass. Never modifies test files. |\n\nThis prevents agents from writing brittle tests or changing them to match broken implementations. Skip only with explicit reason noted.\n\n## Monitoring\n\nMesh events push to you via channel notifications. React to:\n\n- **`job.completed` on mesh** \u2192 read the summary, dispatch gate subagent for that workspace\n- **Agent blocked** \u2192 read the blocker. Route to another agent, escalate to human, or unblock directly\n- **Cross-agent request** \u2192 create a job and dispatch. Agents don't call each other; you route.\n- **Stale agents** \u2192 no heartbeat > 30min? Check tmux pane, investigate\n\n## Dispatch Protocol\n\n**Always invoke dispatch-portable.sh directly. Never reinvent.** The CLI is\na thin Connect-JSON client of helmd's `ship.ops.v1.AgentsService/Dispatch`\nRPC \u2014 every consumer (the helm SPA, the MCP dispatch_job tool, an operator\nat the shell) is a client of the same one path.\n\n```bash\nbash skills/spawn-agent/scripts/dispatch-portable.sh \\\n  --slug <name> --agent <agent> --spec <mission scratch>/job-spec-<name>.md\n```\n\n`dispatch-portable.sh` boots helmd locally if it isn't already running\n(looks for `target/debug/helmd` relative to the repo root, else `helmd` on\nPATH), so it works the same from a fresh shell. Override the daemon URL via\n`HELMD_URL=http://host:port` (or the shorthand `HELMD_ADDR=host:port`).\n\n**Do NOT pass `--no-open`.** It suppresses helmd's terminal-opening logic\nand leaves the user blind \u2014 they cannot see the agent run. helmd\nauto-detects terminal (`$TMUX`, `$WT_SESSION`, `$TERM_PROGRAM`) and opens\na visible window. If the user can't see the agent, they can't observe the\nwork.\n\nIf the user explicitly sets `SHIP_DEFAULT_TERMINAL=manual`, respect it \u2014\notherwise let detection run.\n\n**Parallel dispatches:** issue dispatch-portable.sh calls in the same\ntool-batch when jobs are independent \u2014 worktree/branch isolation handles\nconcurrency. `--base` is accepted but advisory-only; helmd applies its own\n`job/<slug>` convention.\n## Gate Protocol \u2014 verify, don't trust\n\n**An agent's tail-pane summary is not evidence.** \"\u2705 all green\" in chat is a claim. Until you've *run* the acceptance commands, the job is unverified.\n\n**Land is the only merge path.** The integration branch (`feat/next` for the\ncurrent cycle) is the only place agent work lands \u2014 never raw `git merge\n--no-ff job/<slug>` onto `main`, never a local-only squash by hand. Land is\nthe helmd `GitService/Land` RPC: it squash-merges the source branch onto\nthe integration branch, deletes the source branch (it is LANDED by\ndefinition), and removes the worktree if clean. A dirty worktree is left\nin place \u2014 uncommitted work we'd be wrong to discard.\n\n```bash\n# Find the integration branch (ask the operator or check the worktree you\n# were dispatched from \u2014 never assume).\ngit -C <main-repo> rev-parse --abbrev-ref feat/next\n\n# Land the job. The RPC is the merge, the cleanup, and the audit log line.\nship ops call git.land --json '{\n  \"repo_path\": \"<main-repo>\",\n  \"source_branch\": \"job/<slug>\",\n  \"target_branch\": \"feat/next\",\n  \"message\": \"<slug> \u2014 <one-line goal>\"\n}'\n# On a clean Land, the response carries the new integration-branch SHA and\n# the source branch + worktree are already gone. On a conflict, Land\n# reports the conflicting paths and writes nothing \u2014 surface them and\n# resolve by hand.\n```\n\nFor each completed job, before you call Land:\n\n1. **Check git state.** `git -C <wt> status --short` +\n   `git -C <wt> log --oneline feat/next..HEAD` +\n   `git -C <wt> diff feat/next --stat`. Agents frequently:\n   - Never commit (working-tree dirty, no commits ahead of the integration\n     branch) \u2014 you commit on their behalf\n   - Create or write into a `.artifacts/` dir \u2014 the concept is RETIRED;\n     delete the dir, working files belong in the mission's scratch\n   - Touch files outside their spec scope \u2014 flag, decide to keep or revert\n2. **Run the acceptance commands from the spec.** Not \"did the agent claim\n   they passed\" \u2014 run them yourself. `cargo check --workspace --features\n   unstable`, `bun run --filter @ship/<x> test`, the named e2e specs, etc.\n3. **Smoke the user-facing loop** for jobs that change UX/CLI behaviour.\n   Path-convention mismatches (e.g. `page.sui` vs `index.sui`) compile\n   clean but break the actual loop. A 30-second `tmpdir + ship init + curl`\n   is cheap insurance.\n4. **Trivial work** (docs, config, comment-only) \u2014 eyeball diff suffices.\n5. **Heavyweight gates** (feature/schema/API) \u2014 dispatch a `gatekeeper`\n   subagent scoped to the job-branch diff. Act on its verdict.\n\nOn **PASS**: call `git.land` (see above). Land does the squash, branch\ndelete, and worktree cleanup in one RPC \u2014 no separate `git worktree remove`\nor `git branch -D` step. Drop the agent's `.gitignore` modification if\npresent before Land.\nOn **FAIL**: surface specific failures with file:line. Either fix in the\nworktree yourself (small) or file a follow-on spec (large).\n\n## Known Dispatch Pitfalls\n\nThese recur. Watch for them every gate:\n\n- **A `.artifacts/` dir appears** \u2014 the concept is retired; nothing writes\n  there. Delete it and re-point the agent at its mission scratch.\n- **Agent never commits.** Working tree dirty, `git log feat/next..HEAD`\n  empty. You commit on their behalf with a `feat/fix/refactor/...` prefix\n  per CLAUDE.md.\n- **No attribution in commits.** No `Co-Authored-By:` lines. Ever.\n- **Spec acceptance criteria misread.** \"Tests pass\" \u2260 user loop works.\n  Always smoke when the loop changes.\n- **Stale stashed-binary warnings during dispatch compile.** \"migration X\n  was previously applied but missing in resolved\" is the symptom \u2014\n  non-blocking, fix with `just dev-restart` per affected worktree.\n- **Agent flags out-of-scope findings.** Read them. They often catch the\n  next gap (e.g. resolver convention mismatch).\n\n## Cross-Cutting Rules\n\n- **You don't do specialist work.** If you're tempted to edit source files,\n  dispatch an agent instead.\n- **Scope is authority.** `file_scope=\"apps/gateway/\"` means that agent\n  cannot touch `crates/`. Enforce at dispatch and gate.\n- **Escalate what you can't resolve.** Never let a blocker sit silent.\n- **Log decisions, not activity.** Use `log_progress` for state changes\n  and blockers. Not \"checking status.\"\n- **End every session with a handoff.** What was accomplished, what's in\n  flight, what's blocked, what's next. No handoff = next session starts\n  blind.\n\n## Agent Selection\n\n| Work type | Agent |\n|-----------|-------|\n| Rust (runtime, compiler, CLI, MCP) | `rust-lane` |\n| Web / React / Studio | `web-lane` |\n| Design / components / themes | `ship-designer` |\n| Tests only (no impl) | `test-writer` |\n| Code review / gate | `gatekeeper` |\n| Infra / dispatch / scaffold | `shipwright` |\n\nCompose with `--with` for domain knowledge: `ship use rust-lane --with better-auth`, `ship use web-lane --with cloudflare`.\n",
});

defineSkill({
  frontmatter: {
    "name": "code-review",
    "audience": "internal",
    "stable-id": "code-review",
    "description": "Use when reviewing a branch diff. File findings as claims (kind: observation / bug / annotation); the operator triages via the claim inbox. No artifact files \u2014 the new world finds, claims, clears.",
    "tags": ["review", "code-quality", "security"],
    "authors": ["ship"],
  },
  assets: [
    { kind: "asset", path: "skills/code-review/assets/vars.json" },
  ],
  body: "# Review Workflow\n\nStructured code review for branch diffs. The output is **claims**, not\nartifact files \u2014 the operator triages the inbox, the gate reads the\nverdict, and nothing is committed to a `.artifacts/` review-map.md (the\nartifacts concept is retired; everything goes through Helm).\n\n## Protocol\n\n### 1. Read the diff\n\n```bash\n# Branch diff against the integration branch (NOT main \u2014 agents land\n# onto feat/next; you review the same diff the cmdr will Land).\ngit -C <main-repo> diff feat/next..job/<slug> --stat\ngit -C <main-repo> log feat/next..job/<slug> --oneline\ngit -C <main-repo> diff feat/next..job/<slug>\n```\n\n### 2. Classify changes\n\nBuild a change map (in your head or in scratch \u2014 NOT in a committed file):\n\n| File | Type | Risk | Reviewer focus |\n|------|------|------|---------------|\n| `src/auth.ts` | feature | high | security, auth flow |\n| `src/utils.ts` | refactor | low | behavior preservation |\n| `migrations/001.sql` | schema | high | data safety, rollback |\n\n### 3. Review each file\n\nFor each changed file, check in order:\n\n**Security (file as `bug` claim):**\n* SQL injection via string interpolation\n* XSS via unescaped user input\n* Secrets in code or config\n* Auth bypass (missing middleware, unchecked permissions)\n* LLM trust boundary violations (user input \u2192 system prompt)\n\n**Correctness (file as `bug` claim):**\n* Logic errors, off-by-one, null handling\n* Error paths that swallow failures silently\n* Race conditions in async code\n* Missing cleanup (open handles, temp files)\n\n**Tests (file as `observation` claim if missing):**\n* New behavior has test coverage\n* Bug fixes have regression tests\n* Edge cases covered (empty, null, boundary)\n\n**Architecture (file as `observation` claim if drift):**\n* Changes follow existing patterns\n* No unintended coupling introduced\n* File scope appropriate (agent didn't stray)\n\n**Style (non-blocking; usually an `annotation` claim):**\n* Naming conventions\n* Dead code\n* Unnecessary complexity\n\n### 4. File the findings as claims\n\nFor each finding, use the `file_claim` MCP tool (or\n`ship ops call claims.file_claim`). The diff is the citation; the claim\nis the message.\n\n```bash\n# A blocking security finding\nship ops call claims.file_claim --json \"$(cat <<EOF\n{\n  \"kind\": \"bug\",\n  \"target\": {\"subsystem\": \"<name>\", \"feature\": \"<name>\"},\n  \"title\": \"src/auth.ts:42 \u2014 user input passed directly to SQL query\",\n  \"body\": \"String interpolation in db.query() is an injection vector. Replace with parameterized query. The implementer cannot ship this as-is.\",\n  \"citation\": {\n    \"file\": \"src/auth.ts\",\n    \"line\": 42,\n    \"commit\": \"<job-branch SHA>\"\n  }\n}\nEOF\n)\"\n\n# A non-blocking test gap\nship ops call claims.file_claim --json \"$(cat <<EOF\n{\n  \"kind\": \"observation\",\n  \"target\": {\"subsystem\": \"<name>\", \"feature\": \"<name>\"},\n  \"title\": \"src/utils.ts \u2014 parseDate has no test for invalid input\",\n  \"body\": \"Add a unit test for the null/empty/whitespace path; current coverage is happy-path only.\"\n}\nEOF\n)\"\n\n# A style nit (won't gate)\nship ops call claims.file_claim --json \"$(cat <<EOF\n{\n  \"kind\": \"annotation\",\n  \"target\": {\"subsystem\": \"<name>\", \"feature\": \"<name>\"},\n  \"title\": \"src/api.ts:15 \u2014 unused import Response\",\n  \"body\": \"Trivial; left for the implementer to clean up post-merge.\"\n}\nEOF\n)\"\n```\n\n### 5. Hand off the verdict\n\nOn the gate handoff doc (the job's tracking doc, owned by the cmdr),\nrecord the verdict and link the claim IDs:\n\n```markdown\n## Review verdict\n\n* `bug` claims filed: N (block Land until reconciled)\n* `observation` claims filed: N (inform the implementer; do not block)\n* `annotation` claims filed: N (post-merge cleanup)\n\nClaim IDs: c-\u2026, c-\u2026\n\nGate recommendation: PASS | FAIL\n* If any `bug` claim is unreconciled \u2192 FAIL\n* If only `observation` / `annotation` claims \u2192 PASS with notes\n```\n\n## Notes\n\n* The cmdr is responsible for triaging the claim inbox and deciding\n  whether to Land; you file the findings, they own the decision.\n* The diff is the citation. Never cite a line you haven't read in the\n  current `feat/next..job/<slug>` diff.\n* `gh pr review` is no longer the workflow \u2014 the agent reviews its own\n  branch (not a remote PR), files claims, and the cmdr/operator triages\n  via the claim inbox. If you find yourself reaching for `gh`, stop \u2014\n  the workflow changed.\n",
});

defineSkill({
  frontmatter: {
    "name": "collection-authoring",
    "audience": "public",
    "stable-id": "collection-authoring",
    "description": "Hands-on guide to writing a collection schema for Ship. Covers the schema file shape, field-type selection, filterable/required/indexed flags, naming conventions, testing the schema against the API, and common patterns. Pair with ship-collections (reference).",
    "tags": ["collections", "schema", "extension-authoring", "jsonc"],
    "authors": ["ship"],
  },
  body: "# Collection Authoring\n\nYou write one JSONC file at `.ship/collections/<id>.jsonc`, the daemon picks it up, and you have a typed table with CRUD, query, events, and API access. This skill is the practical companion to `ship-collections` \u2014 read that for concept depth and the full query-operator list. Read this when you are sitting down to design a new collection.\n\n## The one-screen schema\n\n```jsonc\n// .ship/collections/todo.jsonc\n{\n  \"id\": \"todo\",\n  \"name\": \"Todo\",\n  \"fields\": {\n    \"title\":      { \"type\": \"string\",   \"required\": true,  \"filterable\": true },\n    \"status\":     { \"type\": \"enum\",     \"values\": [\"open\", \"doing\", \"done\"], \"filterable\": true, \"default\": \"open\" },\n    \"priority\":   { \"type\": \"enum\",     \"values\": [\"low\", \"normal\", \"high\"], \"filterable\": true, \"default\": \"normal\" },\n    \"notes\":      { \"type\": \"markdown\" },\n    \"due_date\":   { \"type\": \"date\",     \"filterable\": true },\n    \"tags\":       { \"type\": \"array\",    \"filterable\": true },\n    \"archived\":   { \"type\": \"boolean\",  \"filterable\": true, \"default\": false }\n  }\n}\n```\n\nDrop the file in `.ship/collections/`, hit `POST /api/collections/todo/items`, done.\n\n## The authoring checklist\n\n1. **`id` is the handle.** Lowercase, underscore-separated, singular (`todo`, not `todos`). It's the URL slug, the event namespace, and the key views reference in their `<data>` subscriptions. Rename = migration.\n2. **Pick field types deliberately** \u2014 see \u00a7Field types.\n3. **Mark what you filter on.** Every field the user sorts, filters, or searches on needs `filterable: true`. The daemon auto-creates an index; queries stay fast under volume.\n4. **Declare required early.** `required: true` at author time beats adding it later when half your rows violate it.\n5. **Provide `default` for enums and booleans.** It removes a class of `NULL` handling bugs in views and triggers.\n6. **No foreign keys.** If `task.project_id` references a project, store the ID string and look up at query time. Collections don't enforce referential integrity \u2014 this is deliberate. Use a trigger if cascade behavior matters.\n7. **No timestamps in the schema.** `created_at` and `updated_at` are managed automatically. Don't declare them. (They're always returned in item responses.)\n\n## Field types\n\n| Type | When to use | Gotchas |\n|---|---|---|\n| `string` | Short text \u2014 title, slug, external id | Don't use for anything > ~200 chars. Use `markdown`. |\n| `markdown` | Long-form content \u2014 notes, descriptions, body | Rendered by views via the `<sui-markdown>` primitive. Not indexed. |\n| `number` | Numeric \u2014 count, score, price | Stored as double (REAL). No int/float distinction. |\n| `boolean` | Flag \u2014 `archived`, `starred`, `is_public` | Coerced on read; safe to filter with `true`/`false`. |\n| `enum` | Fixed list \u2014 status, priority, type | Values enforced on write. Adding a new value is safe; removing is not. |\n| `date` | Date-only \u2014 due date, published date | ISO 8601 (`YYYY-MM-DD`). Use `datetime` if you need time-of-day. |\n| `datetime` | Timestamp \u2014 event time, snapshot time | ISO 8601 with timezone. |\n| `array` | List of strings \u2014 tags, labels, categories | Filterable via `{ \"$contains\": \"x\" }`. Items are always strings today. |\n| `object` | Structured blob \u2014 metadata, config, nested payloads | Not filterable past the top level. If you need to query inside, promote to its own collection. |\n| `file` | File reference | Stores a file ID; resolve via the files API. |\n\nCollection schemas are declared as JSONC under `.ship/collections/`; the collections query layer validates field operators at runtime \u2014 that is the source of truth for the storage mapping.\n\n## Field-level options\n\n```jsonc\n\"status\": {\n  \"type\":       \"enum\",\n  \"values\":     [\"open\", \"closed\"],   // enum only\n  \"required\":   true,                  // must be present on write\n  \"filterable\": true,                  // indexed; usable in filters\n  \"default\":    \"open\",                // applied if field missing on write\n  \"description\": \"Workflow state\"      // shows up in schema-editor UI\n}\n```\n\n`filterable` vs `indexed`: `filterable` is the author-facing switch. The daemon creates the index automatically for every filterable field \u2014 you don't pass `indexed: true` separately.\n\n## Naming\n\n- `id`: snake_case, singular (`bug_triage`, `daily_log`).\n- Field keys: snake_case, descriptive. Avoid abbreviations (`priority` not `pri`).\n- Avoid names that collide with built-ins: `id`, `created_at`, `updated_at`. The daemon refuses writes using these field names.\n- Built-in collection ids are reserved: `specs`, `adrs`, `notes`, `feedback`, `files`. Don't shadow them.\n\n## Testing your schema\n\n```bash\n# 1. Drop the schema file\n$EDITOR .ship/collections/todo.jsonc\n\n# 2. Seed an item\ncurl -X POST http://localhost:9315/api/collections/todo/items \\\n  -H 'Content-Type: application/json' \\\n  -d '{\"title\":\"ship it\",\"status\":\"open\",\"tags\":[\"launch\"]}'\n\n# 3. Query it\ncurl http://localhost:9315/api/collections/todo/items?filter=%7B%22status%22%3A%22open%22%7D\n\n# 4. Watch the events\nship events tail --filter 'collection.item_*'\n```\n\nIf step 2 returns a validation error, read the message \u2014 the validator names the field and the problem. If the column isn't filterable but you filtered on it, the query returns an error listing the filterable fields.\n\n## Common patterns\n\n### Kanban-style status tracking\n```jsonc\n\"status\": { \"type\": \"enum\", \"values\": [\"backlog\", \"in_progress\", \"review\", \"done\"], \"filterable\": true, \"default\": \"backlog\" }\n```\n\n### Soft delete\n```jsonc\n\"archived\": { \"type\": \"boolean\", \"default\": false, \"filterable\": true }\n```\nQueries filter `archived: false` to hide without delete. Add a trigger on `collection.item_updated` to cascade effects if needed.\n\n### Tag/category\n```jsonc\n\"tags\": { \"type\": \"array\", \"filterable\": true }\n```\nFilter by `{ \"tags\": { \"$contains\": \"launch\" } }`.\n\n### Assignee / owner\n```jsonc\n\"assignee_id\": { \"type\": \"string\", \"filterable\": true }\n```\nStore the actor id. Resolve to a display name at query time in the view.\n\n### Aggregate source reference\n```jsonc\n\"source_event\": { \"type\": \"string\" }\n```\nFor items derived by a trigger \u2014 store the triggering event id so you can trace back.\n\n## Schema evolution\n\n- Adding a new field: safe. Existing rows return `null` for the new field until written.\n- Adding a new enum value: safe.\n- Removing a field: the column stays, just stop writing to it. True removal is a migration.\n- Changing a field's type: not supported in-place. Create a new field, migrate with a script, delete the old.\n- Renaming the collection: migration. Don't.\n\n## Virtual collections (read-only external data)\n\nIf the data lives in another system (GitHub issues, Jira tickets) and you don't want Ship to own a copy, see `ship-collections` \u00a7Virtual. The schema file has a `virtual: true` flag, a `connection` reference, and field `map:` expressions \u2014 the query API translates filters to the external API. Out of scope for local authoring.\n\n## When NOT to make a new collection\n\n- You need to store arbitrary short-lived state for a view \u2192 use view preferences (localStorage) instead.\n- You need a per-user setting \u2192 use `/api/settings`.\n- You need to cache an external query result \u2192 that's a virtual collection, not a local one.\n- You need one row with global config \u2192 a kv table fits better; don't spin up a collection for a single row.\n\n## Relation to other systems\n\n- **Views** consume collections by subscribing to `ship.collection.items(id, filter?)` via a `<data>` element in the template. The signal stays live as items change. See `ship-views` for the `<data>` shape and the SDK methods.\n- **Triggers** react to `collection.item_created`/`_updated`/`_deleted` events. See `trigger-authoring`.\n- **Hooks** (agent/provider layer) have nothing to do with collections.\n",
});

defineSkill({
  frontmatter: {
    "name": "configure-agent",
    "audience": "internal",
    "stable-id": "configure-agent",
    "description": "Use when setting up a workspace for a specialist agent \u2014 selecting agent, permission tier, scope, and worktree path. Ensures agents are productive (not over-restricted) and safe (not under-restricted).",
    "tags": ["configuration", "agents", "permissions", "worktree"],
    "authors": ["ship"],
  },
  body: "## Permission Preset Selection\n\nPick the preset that matches the blast radius of the work, not your anxiety level.\nOver-restricting kills productivity \u2014 every approval interruption breaks flow.\n\n| Preset | default_mode | Use for |\n|--------|-------------|---------|\n| `ship-readonly` | `plan` | Reviewer/gate agents, analysis-only, no writes |\n| `ship-standard` | `default` | Interactive sessions, commander, orchestration |\n| `ship-autonomous` | `dontAsk` | Most specialist workers in worktrees \u2014 web, Rust, docs, tests |\n| `ship-elevated` | `dontAsk` | CI agents, release automation \u2014 unlocks push/publish |\n\n**Default to `ship-autonomous` for dispatched agents.** Agents in worktrees doing scoped feature work should not be interrupted. Use `ship-standard` for interactive sessions where you want confirmation prompts.\n\n## Base Rules (all presets)\n\nEvery preset inherits these rules \u2014 they cannot be overridden by agents:\n\n- **Always allow:** `Bash(ship *)` (the Connect op surface \u2014 `ship ops call`)\n- **Always deny:** `Bash(sqlite3 ~/.ship/*)`, `Bash(git push*)`, `Bash(*publish*)`, secrets files (`.env`, `credentials*`)\n- **Always ask:** `.ship/` writes\n\n`ship-elevated` unlocks `Bash(git push*)` and `Bash(*publish*)` via `tools_allow_override`.\n\n## The Compound Command Problem\n\nClaude Code matches permission patterns against the FULL command string.\n`Bash(ship exec*)` will NOT match `cd .target/bin && ship exec blah`.\n\nRules:\n- Never try to allowlist compound commands with patterns \u2014 it won't work\n- Use `default_mode` via presets to set the baseline, `tools_ask` only for specific destructive ops\n- If a legitimate workflow always chains commands, document the full pattern OR grant the preset that doesn't need to ask\n\n```toml\n# Wrong: trying to pattern-match compound commands\ntools_ask = [\"Bash(ship exec*)\"]   # misses: cd dir && ship exec\n\n# Right: set mode, guard only what's genuinely dangerous\n[permissions]\npreset = \"ship-autonomous\"\ntools_deny = [\"Bash(rm -rf*)\"]\n```\n\n## Agent \u2192 Preset Mapping\n\n| Agent | Recommended preset | Rationale |\n|---------|-------------------|-----------|\n| commander | `ship-standard` | Orchestration, needs confirmation for dispatch |\n| gatekeeper | `ship-standard` | Gate + review, needs bash for checks, owns merge |\n| rust-lane | `ship-autonomous` | Scoped feature work in worktree |\n| web-lane | `ship-autonomous` | Scoped feature work in worktree |\n| test-writer | `ship-autonomous` | Tests only, low blast radius |\n| default | `ship-standard` | Interactive human-paired sessions |\n\n## Scope Constraints\n\n\nAlways set explicit file scope in the job spec. The agent constrains capability\n(what the agent knows how to do). Scope constrains authority (what it's allowed to touch).\n\n```\nScope: apps/gateway/src/routes/, apps/gateway/src/lib/\nOff-limits: crates/, apps/mcp/, apps/ship-studio-cli/\n```\n\nAgents WILL respect explicit scope instructions. Don't rely on preset alone.\n\n## Checklist Before Starting an Agent\n\n- [ ] Correct agent for the domain?\n- [ ] Permission preset matched to blast radius (default to ship-autonomous for worktrees)?\n- [ ] Job spec includes scope + off-limits?\n- [ ] Acceptance criteria written (not just \"make it work\")?\n- [ ] Dependencies noted (what must already be true)?\n\n**Worktree + agent setup are no longer checklist items.** helmd's\n`ship.ops.v1.AgentsService/Dispatch` provisions the worktree, branch,\nand agent config in one RPC. The dispatch-portable.sh CLI (or the MCP\n`dispatch_job` tool, or the Launch UI) is the operator-facing entry\npoint; from this skill's perspective the worktree path and the\n`ship use <agent>` step are helmd's job, not yours.\n",
});

defineSkill({
  frontmatter: {
    "name": "gate",
    "audience": "internal",
    "stable-id": "gate",
    "description": "Gate a completed job branch before merge. The objective bar is machine-proven via `just gate`; the operator judges only taste. Run inline or as subagent.",
    "tags": ["gate", "review", "quality"],
    "authors": ["ship"],
  },
  body: "# Gate Review\n\nA merge gate has two layers. **Do not conflate them.**\n\n1. **Objective bar \u2014 machine-proven, not claimed.** `just gate` runs the full check/lint/drift/test surface. If it's green, the mechanical bar is met. You never hand-grep what a command can prove.\n2. **Taste bar \u2014 human/agent judgment.** The one thing no command catches: is this the right abstraction, thin enough, idiomatic, did it solve the right problem. A diff *skim* for shape, not a line-read for correctness.\n\nA gate is PASS only when **both** pass. Green-but-ugly is a FAIL.\n\n## Protocol\n\n```\n1. Read the job spec (acceptance criteria + file scope).\n2. Run `just gate` in the worktree. It MUST exit 0.\n   - It composes: check + lint (clippy + check-raw) + gen-*-check drift + workspace tests.\n   - Never trust the agent's \"tests pass\" \u2014 run it yourself.\n3. Verify the spec's explicit acceptance criteria the gate doesn't cover\n   (e.g. \"no sqlite types in the crate public surface\", \"old module untouched\",\n   additive-only). Grep for them.\n4. TASTE SKIM the diff: right abstraction? thin? matches codebase idiom?\n   over-built? a parallel implementation that should have replaced something?\n5. Both green \u2192 PASS. Either fails \u2192 FAIL with specific evidence.\n```\n\n### Locked-test discipline (the trust anchor)\n\nA passing test the **implementer wrote** proves only self-consistency. The trustworthy signal is a test written by **not-the-implementer** (test-writer agent or operator) that the implementer **could not edit**. When a spec ships locked tests, \"green\" means \"met a bar it couldn't move.\" When it doesn't, weight the test result less and skim the impl harder.\n\nIf a refactor *replaces* an old impl, the new crate's coverage must reach **parity with what the old tests covered** before the old is deleted \u2014 deleting tested code for under-tested code is a FAIL even if green.\n\n## Verdict format\n\n```\nGATE PASS: <job> \u00b7 job/<slug>\n  \u2713 just gate green (check + lint + drift + tests)\n  \u2713 <acceptance criterion> \u2014 <evidence>\n  \u2713 taste: <one line \u2014 abstraction/idiom judgment>\n  Ready to merge.\n\nGATE FAIL: <job> \u00b7 job/<slug>\n  \u2713 just gate green\n  \u2717 <criterion> \u2014 expected X, got Y \u00b7 <command output>\n  \u2717 taste: <what's wrong \u2014 over-built / parallel-impl / wrong seam>\n  What needs to change: <actionable>\n```\n\n## The merge (on PASS) \u2014 Land, never raw git merge\n\nThe merge is owned by helmd's `GitService/Land` RPC. **No raw `git merge`.**\nA raw merge can never hit the integration branch correctly (it would have\nto know the operator's current integration target, and a manual\n`git worktree remove` + `git branch -D` would race with helmd's own\ndispatch lifecycle). Land is the one path that squash-merges onto the\nintegration branch, deletes the source branch (it is LANDED by\ndefinition), and removes the worktree if clean \u2014 all atomic, all logged.\n\n```bash\n# Resolve the integration branch (ask the operator or check the worktree\n# you were dispatched from \u2014 never assume `main`).\nTARGET=$(git -C <main-repo> symbolic-ref refs/remotes/origin/HEAD 2>/dev/null \\\n    | sed 's@^refs/remotes/origin/@@' || echo \"feat/next\")\n\n# Land the job onto the integration branch. On a clean Land the source\n# branch + worktree are already gone; on conflict the RPC reports the\n# conflicting paths and writes nothing.\nship ops call git.land --json \"$(cat <<EOF\n{\n  \"repo_path\": \"<main-repo>\",\n  \"source_branch\": \"job/<slug>\",\n  \"target_branch\": \"$TARGET\",\n  \"message\": \"<slug> \u2014 <one-line goal>\"\n}\nEOF\n)\"\n\n# Re-verify on the merged tree (the integration branch tip).\njust gate\n```\n\nTarget branch is the **CURRENT integration branch** (typically `feat/next`),\nnot `main`. `main` is a release cut, not an integration point. The\n`Release` RPC (separate from `Land`) is the squash that promotes\nintegration \u2192 main; the operator runs it at release time, not at gate\ntime. Record the verdict in the job's tracking doc after Land.\n\n**Why no `git worktree remove` + `git branch -d` here:** Land does the\ncleanup itself as part of the same RPC. Doing it by hand first is a race\n(the cmdr might be dispatching another job into that worktree) and is\nalmost always wrong \u2014 Land's cleanup is gated on a CLEAN worktree, so\nuncommitted work is preserved exactly once.\n\n## Replacement discipline (cutover, not accretion)\n\nThe #1 agentic failure is building-over instead of replacing. An additive\nnew crate is only the FIRST half of a move; the cutover that deletes the\nold impl is the committed second half. When gating a service rewrite:\n- New impl proven (parity tests green) \u2192 file a `kind: annotation` claim\n  (non-blocking) or `kind: observation` (must-fix) against the affected\n  Subsystem/Feature. The deprecation is tracked in the claim inbox, not\n  in a `.artifacts/services/_deprecations.md` file (the artifacts concept\n  is retired \u2014 everything goes through Helm).\n- A FAIL if the diff *adds* a third way to do something instead of\n  replacing.\n\n## Banned-vocabulary gate\n\nThe banned-vocabulary gate is a regression fence against **deleted concepts**\ncrawling back into the tree via a stale test or a careless edit. It is wired\ninto `just lint-vocab` (folded into `check-raw` \u2192 `lint` \u2192 `gate`) and runs\n`scripts/lint/check-banned-vocab.sh`, which ripgreps every banned term across\nhand-authored source, proto, and **tests** (tests are deliberately in scope \u2014\na test reintroducing a dead word must fail).\n\n**Ban-list:** `scripts/lint/banned-vocab.txt`. One entry per line:\n\n```\n<regex-or-literal><TAB><reason>\n```\n\nField 1 is a ripgrep pattern (literal or regex); a single TAB separates it\nfrom the reason, which is printed on a hit. `#` comments and blank lines are\nignored. An **empty list is a no-op** \u2014 the gate exits 0 and prints\n`banned-vocab: 0 terms, clean`. On a hit it prints\n`BANNED VOCAB: <term> \u2014 <reason>` with the offending `file:line` and exits\nnon-zero.\n\n**THE RULE \u2014 never ban a term that still compiles.** A refactor step appends a\nterm to the ban-list only *once its last legitimate use is deleted*. The ban is\nthe closing act of an eradication, not the opening one: delete the real uses,\nprove the tree is clean, then add the fence so the concept can't return.\nBanning a live term breaks the build for everyone \u2014 that is a process error,\nnot a caught regression.\n\n## Notes\n- `just gate` is the source of truth for the objective bar. If a check is missing from it, add the check to xtask/scripts/lint and compose it into the recipe \u2014 never hand-grep in the gate instead.\n- UI surfaces still require a fresh `web-qa` artifact (see `web-qa` skill) \u2014 that gate is unchanged.\n- The cmdr skill's Gate Protocol section is the companion to this one; it owns the operator-facing merge step (Land), this skill owns the objective+taste verdict that authorizes it.\n",
});

defineSkill({
  frontmatter: {
    "name": "memento",
    "stable-id": "memento",
    "description": "Record durable decision records \u2014 corrections, operational rules, mistakes, rejected ideas, tooling/dependency choices, product calls. Reinforces on repeat, supersedes on change, surfaces high-signal candidates for promotion. Use to persist what was decided so future agents don't relitigate it.",
    "tags": ["memory", "decisions"],
    "authors": ["ship"],
    "audience": "internal",
  },
  assets: [
    { kind: "script", path: "skills/memento/scripts/memento.py" },
    { kind: "script", path: "skills/memento/scripts/test_memento.py" },
  ],
  body: "> **EXPERIMENTAL \u2014 NOT A PRODUCT SURFACE.** This skill is a local prototype.\n> Its store is a per-machine sqlite file at `~/.ship/memento.db` (override\n> `MEMENTO_DB`) driven by a stdlib Python script. Nothing here syncs, partitions\n> by scope, or rides the substrate. Use it to dogfood the decision-record idea\n> locally \u2014 do not build product features against `memento.py` or its DB shape.\n>\n> **Product home (ratified decision).** Memento's product form is a\n> **Space-scoped `decisions` collection over the substrate**, with\n> `record` / `query` / `supersede` / `audit` as **SDK ops** \u2014 event-sourced,\n> CRDT-merged, synced like every other collection. The local sqlite prototype\n> below is explicitly NOT that; it exists only to validate the shape before the\n> substrate-backed form is built. When the product form lands, this skill\n> retires or becomes a thin client over the `decisions` collection.\n\n## When to record\n\nRecord only when one of these seven triggers fires:\n\n- **correction** \u2014 a human or gate corrected your approach\n- **operational** \u2014 a standing rule about how work gets done here\n- **mistake** \u2014 something that broke or wasted effort; capture so it isn't repeated\n- **rejected** \u2014 an idea considered and explicitly turned down (and why)\n- **tooling** \u2014 a tool/workflow choice that constrains future work\n- **dependency** \u2014 a library/version/service commitment\n- **product** \u2014 a product or UX decision with alternatives\n\nFidelity rule: record **what was decided, stated, or done** \u2014 never inferred intent, never every passing thought. One record per decision. If it isn't a settled fact, don't write it.\n\n## How\n\n`python3 scripts/memento.py <subcommand>`\n\n- **record** \u2014 log a decision; identical kind+title (case-insensitive, active) reinforces instead of duplicating, bumping confirmations and conviction one notch.\n  `record --kind operational --title \"Use WAL mode\" --conviction high --tags db,sqlite`\n- **query** \u2014 pull terse one-liners, ranked by conviction then confirmations.\n  `query --kind tooling --min-conviction medium --tag db --limit 20`\n- **get** \u2014 full record (with body) as pretty JSON by id.\n  `get a1b2c3d4e5f6...`\n- **supersede** \u2014 replace an existing record with a corrected one; links old\u2194new, no dedup.\n  `supersede <OLD_ID> --kind correction --title \"New decision\" --body \"why it changed\"`\n- **audit** \u2014 operator review of active records ranked by confirmations, labeled STRONG candidate / candidate / weak for promotion.\n  `audit --limit 30`\n",
});

defineSkill({
  frontmatter: {
    "name": "ship-canvas",
    "audience": "public",
    "stable-id": "ship-canvas",
    "description": "Use when the conversation needs a visual surface \u2014 mockups, data visualizations, diagrams, design reviews \u2014 where the human wants to watch and guide your work from a single pane. You produce a live Ship view (not a static HTML file) that lets the human send structured feedback back to you via interactive widgets you design.",
    "tags": ["design", "canvas", "visualization", "feedback", "extension-authoring"],
    "authors": ["ship"],
    "artifacts": ["html"],
  },
  assets: [
    { kind: "asset", path: "skills/ship-canvas/assets/base-template.html" },
    { kind: "asset", path: "skills/ship-canvas/assets/vars.json" },
  ],
  body: "# Ship Canvas \u2014 interactive visual surface between agent and human\n\nYou're producing a **live, interactive Ship view** \u2014 not a static HTML\nfile. The human loads it in Studio or the shell, interacts with widgets\nyou design, and their input flows back to you as structured feedback on\nyour next turn.\n\nThis is the single-pane loop: human watches, you build, human rates,\nyou iterate.\n\n**Prerequisite:** Read `ship-views` first. This skill assumes you know\nthe `.sui` template format, the paired `controller.ts` scope, and the\nthree-file shape (`page.sui` + optional `controller.ts` + optional\n`style.css`). This is the canvas-specific guidance on top.\n\n## What to produce\n\nDrop files into `views/canvas-<slug>/`:\n\n- `manifest.jsonc` \u2014 short metadata; usually no data subscriptions.\n- `page.sui` \u2014 template using `<sui-*>` primitives.\n- `controller.ts` \u2014 signals + the `sendFeedback` helper that POSTs to\n  `/api/v1/feedback` with your payload.\n- (optional) `style.css` \u2014 only if a layout primitive can't express it.\n\nThe daemon serves it live at `/canvas-<slug>`. Edit any file \u2192 the\nshell hot-reloads \u2192 human sees the update. No build step, no restart.\n\n## The feedback contract \u2014 you design the payload\n\nThe human's feedback doesn't have to be \"drew a box, added a note.\"\nYou decide what feedback shape makes sense for what you're showing.\nEach interactive control wires to a payload you authored.\n\nWire-format the daemon accepts (POST `/api/v1/feedback`):\n\n```ts\n{\n  id: string,                   // ULID \u2014 generate per click\n  kind: 'action' | 'point' | 'region',\n  target: { view_id?: string, file_path?: string, /* \u2026 */ },\n  payload?: unknown,            // \u2190 YOUR shape\n  note?: string,                // free-text companion, optional\n  created_at: string,\n  author_kind: 'human'\n}\n```\n\n`payload` is opaque to the daemon \u2014 the receiving prompt reads it.\n\n```html\n<!-- page.sui -->\n<sui-cluster gap=\"3\" align=\"center\">\n  <sui-button tone=\"primary\" variant=\"solid\"\n    on:click={() => sendFeedback({ option: 'A', rating: 5 })}>\n    5 stars for A\n  </sui-button>\n  <sui-button variant=\"outline\"\n    on:click={() => sendFeedback({ option: 'A', rating: 3 })}>\n    3 stars for A\n  </sui-button>\n</sui-cluster>\n\n<sui-textarea bind:value={note} placeholder=\"Quick thought\u2026\"></sui-textarea>\n<sui-button tone=\"accent\" variant=\"solid\"\n  on:click={() => sendFeedback({ kind: 'comment', text: note() })}>\n  Send\n</sui-button>\n```\n\n```ts\n// controller.ts\nimport { controller, signal } from '@ship/sui-runtime'\nimport type { Scope } from './page.sui'\n\nexport default controller<Scope>({\n  scope: () => ({\n    note: signal(''),\n    sendFeedback: async (payload: unknown, extra?: { note?: string }) => {\n      await fetch('/api/v1/feedback', {\n        method: 'POST',\n        headers: { 'Content-Type': 'application/json' },\n        body: JSON.stringify({\n          id: crypto.randomUUID(),\n          kind: 'action',\n          target: { file_path: location.pathname },\n          payload,\n          note: extra?.note,\n          created_at: new Date().toISOString(),\n          author_kind: 'human',\n        }),\n      })\n    },\n  }),\n})\n```\n\nEach `sendFeedback(...)` call emits `studio.feedback.created` through\nthe kernel; you read it on your next turn via the events API.\n\n**Design rule:** the payload should be structured enough that you can\nact on it in one prompt. Free-text requires you to parse \u2014 enums and\nnumbers don't. Prefer:\n\n- `{option: 'A', rating: 5}` over `\"rating: A-5\"`\n- `{action: 'approve', section: 'hero'}` over `\"approve hero\"`\n- `{kind: 'copy-direction', preference: 'bold'}` over `\"make it bolder\"`\n\nThe human still has free-text capture (via `bind:value` + a `note`\nfield on the payload) when they want to say something unstructured.\nYou're giving them faster structured buttons for the common responses.\n\n## The single-pane loop\n\nTypical session:\n\n1. Human: \"show me two directions for the hero\"\n2. You: write `views/canvas-hero/{page.sui,controller.ts,manifest.jsonc}` with two mocks + rating buttons\n3. Human: loads `/canvas-hero` in the shell, clicks ratings, types a comment\n4. You: receive `{option: 'A', rating: 4}`, `{option: 'B', rating: 2}`, `{kind: 'comment', text: '\u2026'}`\n5. You: edit the same files \u2014 now three directions, rating buttons preserved\n6. Human: shell hot-reloads, rates again\n7. Iterate until one is approved, then `{action: 'approve', section: 'hero'}`\n\n**Iterate in place.** Past versions live in git history, not as\n`page-v2.sui` files.\n\n**When the human approves**, produce a text reply summarizing what was\nlearned and where the final artifact lives. The human can copy that\ninto wherever it needs to go (a real product view, a landing page,\netc.).\n\n## What to make\n\n- **Mockups** \u2014 UI layouts, pages, components, design explorations\n- **Data visualizations** \u2014 use the `@ship/sui-charts` primitives\n- **Diagrams** \u2014 architecture, relationships, flows\n- **Design reviews** \u2014 side-by-side options, ratings, pick-one\n- **Interactive reports** \u2014 \"scroll through these findings, flag any that look wrong\"\n\nNot a fit: rich interactive editors (canvas games, video timelines),\nanything that needs frames per second. Those belong in a full client\nframework.\n\n## Visual quality\n\nYou're a senior designer who codes. Render like it's production:\n\n- **Use the primitives** \u2014 never hand-code buttons, cards, selects, toolbars. `<sui-*>` covers all of them. See the `ship-components` skill for the catalog.\n- **Use layout primitives** \u2014 `<sui-stack>` / `<sui-cluster>` / `<sui-grid>` / `<sui-center>` instead of bespoke `display:flex` CSS. See the `ship-views` skill.\n- **Use design tokens** \u2014 `var(--space-N)`, `var(--text-base)`, `var(--font-display)`, `var(--foreground)`, etc. No hardcoded px or hex. The compiler enforces this via `sui-css-literal` \u2014 raw colors are a hard error.\n- **Use the canonical `tone` \u00d7 `variant` vocabulary** \u2014 `tone=\"primary\" variant=\"solid\"` for primary CTAs, `tone=\"destructive\" variant=\"soft\"` for destructive, `variant=\"ghost\"` for tertiary. The compiler enforces this via `sui-prop-value` \u2014 unknown values are a hard error.\n- **Realistic content** \u2014 no lorem ipsum, no \"Chart Title\", no \"Label\". Write the actual copy.\n- **Clear hierarchy** \u2014 display font for titles, body font for copy, monospace for code/IDs.\n- **One concept per canvas** unless the human asked for a dashboard. Whitespace > density.\n\n## Template starter\n\n```html\n<!-- views/canvas-hero/page.sui -->\n<sui-center max=\"wide\" padded>\n  <sui-stack gap=\"6\">\n\n    <header>\n      <h1>Hero exploration</h1>\n      <p>Two directions for the hero section. Rate each, drop a note if it helps.</p>\n    </header>\n\n    {#for opt of options() key={opt.id}}\n      <sui-card>\n        <sui-stack gap=\"3\">\n          <h2>{opt.label}</h2>\n          <div class=\"mock\">{@html opt.html}</div>\n          <sui-cluster gap=\"2\" justify=\"end\">\n            {#for n of [1, 2, 3, 4, 5] key={n}}\n              <sui-button size=\"sm\" variant=\"ghost\"\n                on:click={() => sendFeedback({ option: opt.id, rating: n })}>\n                {n}\n              </sui-button>\n            {/for}\n          </sui-cluster>\n        </sui-stack>\n      </sui-card>\n    {/for}\n\n    <sui-stack gap=\"2\">\n      <sui-textarea bind:value={note} placeholder=\"Free-text reaction\u2026\"></sui-textarea>\n      <sui-button tone=\"primary\" variant=\"solid\"\n        on:click={() => sendFeedback({ kind: 'comment', text: note() })}>\n        Send note\n      </sui-button>\n    </sui-stack>\n\n  </sui-stack>\n</sui-center>\n```\n\n## Reading feedback on your next turn\n\nFeedback arrives as `studio.feedback.created` events in the kernel\nstream. Use the events API to pull them:\n\n```bash\nship events list --filter 'type=studio.feedback.created' --since 1h\n```\n\nEach event payload matches the `FeedbackEvent` wire shape \u2014 pull\n`payload` out and act on it. If the human clicked\n`{option: 'A', rating: 5}` and `{option: 'B', rating: 2}`, they prefer\nA \u2014 iterate on A, maybe generate A2/A3 variants.\n\n## Live updates \u2014 when your artifact is long-running\n\nIf your canvas is long-running (watching a job progress, live\ndashboard, collaborative review), wire a `<data>` subscription to a\nlive source (e.g. `ship.events.tail()` or a domain endpoint). The\nruntime keeps the signal current via SSE \u2014 no extra plumbing needed.\n\n```html\n<data name=\"run\" source=\"ship.pipeline.run({slug})\" live />\n```\n\nFor most one-shot design reviews you don't need this \u2014 just write the\nview and iterate by editing the files.\n\n## Rules\n\n- **Production quality.** Every canvas should look finished.\n- **Iterate in place.** Same directory, same files. Git is your history.\n- **Primitives first.** Check the `<sui-*>` catalog before writing CSS.\n- **Tokens only.** The compiler will reject raw hex / rgb / px.\n- **Canonical enum values.** The compiler will reject unknown `tone` / `variant` / `size` strings.\n- **Payload shapes are yours to design.** Pick ones you can act on in one prompt.\n\n## Reference\n\n- `ship-views` \u2014 primary authoring skill (required read)\n- `ship-components` \u2014 component catalog\n- `ship-design-system` \u2014 tokens, themes, surfaces\n- `apps/docs/src/content/docs/views/examples.md` \u2014 worked examples of the three-file shape\n- `crates/core/ship-substrate/src/events/feedback.rs` \u2014 `FeedbackEvent` source of truth\n",
});

defineSkill({
  frontmatter: {
    "name": "ship-components",
    "stable-id": "ship-components",
    "description": "Author Ship components in SUI \u2014 `<sui-*>` primitives compiled AOT from `.sui` source with optional paired `.controller.ts`.",
    "tags": ["ui", "components", "web-components", "sui", "design-system"],
    "authors": ["ship"],
    "audience": "public",
  },
  assets: [
    { kind: "reference", path: "skills/ship-components/references/component-contract.md" },
    { kind: "reference", path: "skills/ship-components/references/packs.md" },
  ],
  body: "# ship-components\n\nA `<sui-*>` component is a `.sui` template plus an optional `.controller.ts`. The compiler emits one ESM per component; importing it triggers `customElements.define()`. Real custom elements, real shadow DOM, no runtime parser.\n\n## The rule\n\n**Templates declare. Controllers do.** Dynamic markup goes in the `.sui` template via `{#if}/{#for}/{#match}` and the binding primitives (`bindAttr`, `bindText`, `bindClass`, `bindValue`, \u2026). Controllers own state, refs, lifecycle, async I/O, and the public API. A controller that produces markup is a bug \u2014 `sui-controller-purity` rejects it at compile and at edit.\n\nForbidden in `*.controller.ts`:\n- `<el>.innerHTML = \u2026` / `<el>.outerHTML = \u2026` / `<el>.insertAdjacentHTML(\u2026)`\n- A sibling `render.ts` of HTML-string template-literal helpers\n- String-selector queries against `shadowRoot` after `mount` (use `bind:this` for refs)\n- A `.sui` body that is just `<div bind:this={ref}></div>` \u2014 the template is the renderer\n\nFor HTML-blob cases (markdown rendering, third-party SVG generators, icon-pack swaps), use `bindHtml(el, () => string)` from `@ship/sui-runtime`.\n\n## File layout\n\n```\npackages/components/sui/primitives/src/<tag>/\n  <tag>.sui                  # template + <prop> + <style>     (required)\n  <tag>.controller.ts        # paired controller               (optional, auto-detected)\n  <tag>.css                  # extracted styles                (optional)\n```\n\nConvention is the contract: directory IS the component, filename stem IS the tag, shadow is always on, paired `.controller.ts` is auto-detected. Plain `.ts` files are not controllers \u2014 they're helpers, types, or tests.\n\n## Authoring shape\n\nFile IS the component \u2014 no `<component>` wrapper. Top-level `<prop>` directives declare the API; the rest is the body.\n\n```html\n<!-- packages/components/sui/primitives/src/sui-button/sui-button.sui -->\n<prop name=\"tone\" values=\"default|primary|destructive\" default=\"default\" reflect />\n<prop name=\"disabled\" type=\"bool\" default=\"false\" reflect />\n<prop name=\"size\" values=\"sm|md|lg\" default=\"md\" reflect />\n\n<style>\n  :host { display: inline-flex; }\n  button { /* \u2026 */ }\n</style>\n\n<button disabled={disabled()} class=\"btn btn-{tone()} btn-{size()}\">\n  <slot name=\"leading\"></slot>\n  <slot reflect-empty=\"data-icon-only\"></slot>\n</button>\n```\n\nBindings, control flow, hooks \u2014 same surface as views. See `ship-views`'s `references/template-surface.md` for the full directive + bindings + control-flow reference (the template surface is shared between views and components).\n\n## Paired controller\n\nTemplates declare; controllers contribute state via the `scope:()` factory.\nThe compiler reads that factory and emits each returned key into the\n`Scope` interface of `<file>.sui.d.ts`, so `mount(state, host)` sees direct,\ntyped members \u2014 no casts, no defensive throws.\n\n```ts\n// sui-error-boundary.controller.ts \u2014 the canonical exemplar.\nimport { controller, signal, attachBoundary, detachBoundary } from '@ship/sui-runtime'\nimport type { Scope } from './sui-error-boundary.sui'\n\nexport default controller<Scope>({\n  scope: (_baseScope, host) => {\n    const caughtError = signal<Error | null>(null)\n    const reset = () => { caughtError.set(null) }\n    return { caughtError, reset }\n  },\n  mount(state, host) {\n    attachBoundary(host, {\n      catch(err) { state.caughtError.set(err instanceof Error ? err : new Error(String(err))) },\n      reset: state.reset,\n    })\n    return () => detachBoundary(host)\n  },\n})\n```\n\nThree things to notice:\n\n- `scope:` returns plain identifiers (`caughtError`, `reset`). Those keys\n  land in template scope AND on the typed `state` arg to `mount`.\n- `state.caughtError.set(err)` \u2014 direct, typed `Signal<Error | null>`. No\n  `state['caughtError'] as unknown as Signal<...>` cast. No\n  `throw new Error('scope signals missing')` defensive guard.\n- If you rename `caughtError` in the factory but forget to update a\n  template reference, the TypeScript compiler catches it.\n\nUse `controller(def)` only \u2014 raw `function setup(ctx)` is not accepted. Get refs via `bind:this={el}` in the template; declarative slot reactivity (`<slot reflect-empty=\"<attr>\">`) replaces hand-rolled slotchange listeners.\n\n**Don't create an empty controller.** Controllers are optional \u2014 if your\ncomponent has no `scope`, no `derived`, no `on`, and no `mount`, skip the\n`.controller.ts` file entirely. The compiler synthesizes a no-op controller\nfrom the `.sui` template alone. The LSP rule `controller.empty` flags\n`controller({})`-shape files (and `export default null`) as a warning;\ndelete them rather than satisfy the convention with ceremony.\n\nWorking exemplars: `packages/components/sui/primitives/src/sui-error-boundary/`,\n`packages/components/sui/primitives/src/sui-portal/`,\n`packages/components/sui/primitives/src/sui-toast-container/`.\n\n## Form-aware components\n\nWhen a component participates in HTML form submission (input, textarea, checkbox, switch, select, file upload, \u2026), mark its value prop with `form-value` and its disabled prop with `form-disabled`:\n\n```html\n<prop name=\"value\" type=\"string\" default=\"\" form-value />\n<prop name=\"disabled\" type=\"bool\" default=\"false\" reflect form-disabled />\n```\n\nThe compiler emits `static formAssociated = true` on the generated host class; the runtime calls `host.attachInternals()` on connect and auto-wires the full platform lifecycle:\n\n- `setFormValue(<value>)` runs on every change to the marked prop.\n- `formResetCallback()` resets the prop to its declared `default`.\n- `formStateRestoreCallback(state)` rehydrates the prop on bfcache restore.\n- `formDisabledCallback(disabled)` syncs the marked disabled prop when the surrounding form's disabled state cascades.\n\nDrop any hand-rolled `ElementInternals` plumbing \u2014 the runtime owns it. The setup ctx surfaces `internals?: ElementInternals` if a controller needs `setValidity()` / `checkValidity()` for custom validation.\n\nWorked examples: `packages/components/sui/primitives/src/sui-input/sui-input.sui`, `packages/components/sui/primitives/src/sui-select/sui-select.sui`, `packages/components/sui/primitives/src/sui-checkbox/sui-checkbox.sui`.\n\nConstraints (compiler-enforced):\n\n- Only one prop per component may carry `form-value`.\n- `form-value` is rejected on `type=\"object\"` / `type=\"array\"` props (only `string` / `number` / `bool` / `union`).\n- `form-disabled` is only valid on `type=\"bool\"` props.\n\n## Compile shape\n\n```js\n// AUTO-GENERATED by sui-compiler \u2014 do not edit\nimport { component, prop, bindAttr, bindText, \u2026 } from \"@ship/sui-runtime\"\nimport setup from \"./ship-terminal.controller.ts\"  // when paired\n\nexport default component(\"ship-terminal\", {\n  props: { /* \u2026 */ },\n  shadow: true,\n  styles: \"\u2026\",\n  setup(ctx) { /* scope construction; calls paired setup if present */ },\n})\n```\n\nAuthors don't call `component()` directly \u2014 the compiler emits it.\n\n## Bundling + registration\n\n1. The daemon's in-process Rolldown bundler (#163) compiles every `.sui` source and bundles `sui/crates/deck/deck-core/assets/sui-components.js` (paired `.controller.ts` resolved + inlined by the bundler's load hook). `@ship/sui-build` produces the same tree at CI time.\n2. The shell loads `sui-components.js` once at boot via the importmap \u2192 all `<sui-*>` elements register globally.\n3. `<sui-button>` and friends are usable in any view.\n\n## Hard rules\n\n1. **`tone` for semantic color, `variant` for visual style.** `variant=\"destructive\"` is wrong \u2014 destructive is a tone.\n2. **Composition over reimplementation.** Any list row / tile / card uses `<sui-list-tile>` (slots: `leading`, default, `meta`, `trailing`). Don't ship a thin wrapper whose only value is filling slots with view-specific content.\n3. **Distribution test.** An element earns a spot in the library only if it's a semantic primitive, a platform concept, or has its own brain. View-local compositions live in `partials/`. See `references/component-contract.md` \u00a7 \"The distribution test.\"\n4. **Icon-only buttons need `aria-label`.**\n5. **`<sui-select>` uses native `<option>`** (no `<sui-option>`). Emits `sui-change`.\n6. **No `el.innerHTML/.outerHTML/.insertAdjacentHTML`** in `*.controller.ts`. Use `bindHtml()` for HTML-blob cases.\n7. **No `customElements.define` in component code** \u2014 the compiler emits the registration.\n8. **`{#for}` blocks declare `key={EXPR}`.** Without a stable key the reconciler falls back to positional matching and DOM identity is lost on reorder. `sui-for-key` lint warns when missing.\n9. **`on:<event>` event names must match the platform whitelist:** DOM events, `sui-*` / `ship-*` custom-event families, `tag:event`-prefixed cross-component events. `sui-event-name` lint warns on typos with Levenshtein suggestions.\n10. **Form-aware components use `form-value` / `form-disabled` on `<prop>`** \u2014 never hand-roll `attachInternals()` / `setFormValue` / `formResetCallback`. See \"Form-aware components\" above.\n\n## Verify before \"done\"\n\n**The gate is the compiler, and it is always available.** Run\n`sui-compiler build <tag.sui | component-dir>` after every edit \u2014 it ships on\nPATH (`~/.cargo/bin/sui-compiler`, `just install-dev`), needs no MCP and no\ndaemon, and runs grammar + semantic checks **and the full lint pass** (the\noxc-based controller lint in `sui/crates/sui/lint`, e.g. `sui-controller-purity`,\n`sui-css-content-text`). A printed `Error:` is a hard fail. Never substitute\n`tsc` \u2014 Ship does not bundle TypeScript; the compiler's lint is the source of truth.\n\n```sh\nsui-compiler build packages/components/sui/primitives/src/sui-X/sui-X.sui\n```\n\n## Agent surfaces (CLI)\n\n> The MCP tools below have been **removed** in favour of generated\n> skills + CLI. They wrapped the **same** analyser as `sui-compiler build`, so they\n> added nothing the gate above doesn't already cover \u2014 use the CLI equivalents listed here.\n\nThe view-authoring tools all apply to components since components\nare just `<sui-*>` views with shadow DOM. Full catalog:\n`docs/ship-sui/references/docs/agent-surfaces.md`.\nThe component-specific essentials:\n\n| Tool | When |\n|---|---|\n| `ship sui diagnostics <file>` | Structured parse + analyse. Catches parse errors, missing `<prop>` / `<state>` attrs, unbound identifiers, invalid template expressions. |\n| `ship sui hover <file> <line> <col>` | Declaration source + setup return type at a cursor. |\n| `ship sui dependencies <file>` | What sibling `<sui-*>` tags + signals this component reaches into. |\n| `sui-compiler build <file>` | Full lowering pipeline. Run after `ship sui diagnostics` is clean to catch lowering-only issues. |\n\n### Visual + behavioural verification\n\n| Surface | Use it for |\n|---|---|\n| `ship-ui-studio` at `/sui/ship-ui-studio/<tag>` | Single-page visual sweep \u2014 prop panel + variant matrix + source view. Fastest after-edit eyeball. |\n| `@ship/sui-eval` `runEval({ description, spec, catalog })` | Programmatic behavioural scoring. Sub-scores: `compile / scope / behaviour`. See `docs/ship-sui/references/docs/agent-surfaces.md` \u00a7 3. |\n\n```ts\n// Pseudocode of the verify loop:\n// 1. write template + (optional) controller\n// 2. ship sui diagnostics packages/components/sui/primitives/src/sui-X/sui-X.sui\n//    \u2192 fix until []\n// 3. sui-compiler build packages/components/sui/primitives/src/sui-X/sui-X.sui\n//    \u2192 fix lowering-only issues\n// 4. Open ship-ui-studio for visual sweep across the variant matrix\n// 5. (optional) runEval against a behavioural spec for regression gating\n```\n\n## LSP integration\n\nThe same analyser that backs `sui_diagnostics` is exposed as a\nstandalone language server.\n\n| Surface | How |\n|---|---|\n| **CLI** (agents) | `ship sui diagnostics` / `hover` / `dependencies`. See above. |\n| **VS Code** | `apps/vscode/` auto-launches the `sui-lsp` binary on `.sui` open. |\n| **Any editor / Claude Code / opencode** | Point the LSP client at the `sui-lsp` binary (`sui/crates/sui/lsp/`, on PATH via `just install-dev`) over stdio. Claude Code uses the repo `.lsp.json`. |\n| **Browser** | `@ship/sui-lsp-browser` runs the WASM `analyze` (same Rust grammar) for diagnostics in the docs playground; hover/completion are static-dictionary for now. |\n\n## What's available in a view\n\nComponents fall into these buckets:\n\n- **Primitives** \u2014 generic `<sui-*>` in **`@ship/sui-primitives`** (`packages/components/sui/primitives/src/`). Charts in **`@ship/sui-charts`** (`packages/components/sui/charts/src/`). Both are decomposed workspace packs with their own bundle + `custom-elements.json`. Always registered, available in any view.\n- **Editor packs** \u2014 `<ship-code-editor>`, `<ship-terminal>`, and the `<ship-git-diff>` tree live in **`@ship/code`** (`packages/components/ship/code/src/`); `<ship-markdown-editor>` lives in **`@ship/markdown`** (`packages/components/ship/markdown/src/`).\n- **Chrome + domain `<ship-*>`** \u2014 picker, nav, devtools, view-authoring, and the rest live in their own `ship/*` packs under `packages/components/ship/<pack>/src/` (e.g. chrome in **`@ship/chrome`**, agents in **`@ship/agents`**). Each pack ships its own bundle + `custom-elements.json`.\n- **Islands** \u2014 heavy components that lazy-load their engine only when mounted (ship-terminal, ship-code-editor, ship-markdown-editor). The bundle code-splits these; lazy chunks live under `.deck/sui-components/<tag>.js` (deck's build output).\n\nThe `@ship/sui-catalog` walker emits two projections under `.deck/catalogs/`:\n- `workspace.json` \u2014 every pack, public + private. Use for authoring.\n- `published.json` \u2014 `!private` packs only. Use when generating consumer-facing docs or npm publish flows.\n\nEach entry gains a `pack` field \u2014 filter by pack to scope a query. Read these projections from `.deck/catalogs/` directly. Don't `curl` the daemon's `/api/assets/sui-custom-elements.json` \u2014 that endpoint exists for browser tooling; agent auth blocks raw HTTP.\n\n## Visual sweep: ship-ui-studio\n\nOnce a component is registered, `ship-ui-studio` is the fastest way to eyeball it. Open `/sui/ship-ui-studio/<tag>` (e.g. `/sui/ship-ui-studio/sui-button`) and the detail page renders:\n\n- A **single live preview** plus a prop panel for ad-hoc tweaking.\n- A **state variant matrix** \u2014 when the component declares enum props, every cell in the cartesian product of `variant \u00d7 tone` (or the single largest enum, if there's only one) is rendered side-by-side, independent of the prop panel. This is the Storybook-style sweep that catches \"ghost tone-warning has wrong contrast\" without clicking through 40 combinations one at a time.\n- A **View source** link in the header that opens `/_deck/source/<tag>` \u2014 a static HTML page showing the on-disk `.sui` template and `.controller.ts` for the component. Useful when reviewing how an existing primitive wires its scope before authoring a similar one.\n\n## Platform features (shared with views)\n\nComponents live inside the same runtime as views; the same primitives are\navailable. Skim the matrix below; for deep treatment of each, see the\n`ship-views` skill section of the same name.\n\n### Morph\n\nAdd the `morph` attribute to any element whose contents are produced by\n`bindRawHtml` (or another HTML-string path) to preserve focus, scroll,\nselection, and event-listener identity across re-renders. `bindHtml`\nmorphs unconditionally \u2014 the directive only matters for `bindRawHtml`.\n\n```html\n<div morph>{/* server-rendered HTML \u2014 bindRawHtml in the controller */}</div>\n```\n\nLimitations: no `<select>`-value preservation, no shadow-DOM matching,\nmatches by `id` and tag-plus-position only.\n\n### Error boundaries\n\nWrap fault-prone subtrees in `<sui-error-boundary>` \u2014 throws inside\neffects, controller `mount`, or render expressions are caught and the\nfallback renders. `fallback-text` for a string fallback or\n`<template slot=\"fallback\">` for custom markup. Call `.reset()` on the\nhost to retry. Fires `sui-error-boundary:caught` (bubbling, composed)\nwith `{ error, source: 'effect' | 'mount' | 'render' | 'unknown' }`.\n\n```html\n<sui-error-boundary fallback-text=\"Couldn't load\">\n  <sui-some-component prop={maybeThrows()}></sui-some-component>\n</sui-error-boundary>\n```\n\n### Portal\n\n`<sui-portal target=\"body\">` teleports slotted content out of the host\nshadow tree for stacking-context escape (modals, popovers, toasts).\n`target` is a CSS selector resolved against `document`; default `body`.\nThe portal element stays the logical parent, so reactive bindings and\nevent bubbling still flow through the original DOM ancestry. Missing\ntarget warns and leaves children in place. See `sui-dialog`,\n`sui-popover`, `sui-toast-container` for composition patterns.\n\n### Transitions\n\n`transition:<name>` animates enter/exit of `{#if}` / `{#for}` mounts.\nBuilt-ins: `fade`, `slide`, `scale`. Use `in:` / `out:` for asymmetric\ndirections. Custom transitions live on the controller's `scope:` factory\nand match the `Transition` type. `prefers-reduced-motion: reduce`\ncollapses duration + delay to `0`. Transitions don't surface in SSR \u2014\nthey register on construction (CSR + hydrate paths).\n\n```html\n{#if open()}\n  <div transition:scale={{ duration: 120 }}>...</div>\n{/if}\n```\n\n### Routes\n\nURL state is reactive. Components rarely consume routes directly \u2014 that's\na view-controller concern \u2014 but a component controller may read\n`route()` to surface URL-dependent behavior (e.g. a tab strip whose\nactive tab is `route().query.tab`).\n\n```ts\nimport { route, defineRoutes, navigate } from '@ship/sui-runtime/route'\n// route().params.<name> / route().query.<name> are Signal<string | undefined>\n```\n\nFull treatment: `ship-views` \u00a7 Routes.\n\n### HMR\n\nIn dev mode, `.sui` and `.controller.ts` edits trigger an in-place re-mount.\nshell's boot calls `initHmr()` (`apps/shell/src/hmr.ts`), which uses\n`subscribeHmr` from `@ship/sui-runtime/dev` to listen on shipd's\n`/_deck/dev/hmr` SSE channel. Component-tag changes still need a full\nreload (custom-element re-registration isn't yet supported); compile\nerrors surface in the browser console. Production builds inline\n`import.meta.env.DEV` to `false`, so HMR is a no-op.\n\n### Performance gates\n\n- `just bench-check` \u2014 js-framework-benchmark vs latest baseline under\n  `tests/perf/baselines/`; fails on >15% regression. Run before merging\n  runtime-affecting component changes.\n- `just visual` \u2014 Playwright screenshots vs committed PNGs (daemon must\n  be running). Use `--update-snapshots` after a deliberate visual change.\n- `just smoke-deploy` \u2014 full build + daemon restart + every baselined\n  view curled + Playwright. Proxy for CI; aim for green before PR.\n\n## References\n\n| When you need | Read |\n|---|---|\n| Full component-authoring contract: scope resolution, slot patterns, `bind:this`, the distribution test | `references/component-contract.md` |\n| Component-pack manifests, distribution, on-demand loading | `references/packs.md` |\n",
});

defineSkill({
  frontmatter: {
    "name": "ship-design-system",
    "audience": "public",
    "stable-id": "ship-design-system",
    "description": "Charter for Ship's design-system layer \u2014 tokens, themes, surfaces, tone ramps, motion, density. Load this when authoring or reviewing changes to sui/packages/sui/tokens/src/tokens.css or themes.css, adding a new theme, debugging why a component doesn't respond to a theme switch, or helping a user customize Ship's look beyond brand color.",
    "tags": ["design-system", "tokens", "themes", "css", "theming", "frontend"],
    "authors": ["ship"],
  },
  assets: [
    { kind: "scaffold", path: "skills/ship-design-system/scaffolds/README.md" },
    { kind: "scaffold", path: "skills/ship-design-system/scaffolds/brand-accent.json" },
    { kind: "scaffold", path: "skills/ship-design-system/scaffolds/dark.json" },
    { kind: "scaffold", path: "skills/ship-design-system/scaffolds/high-contrast.json" },
    { kind: "scaffold", path: "skills/ship-design-system/scaffolds/light.json" },
  ],
  body: "# Ship Design System\n\nShip ships a **full design system**, not a color palette.\n\nA theme is not just a swap of `--primary`. A Ship theme can bend surface\ndepth, motion curves, radii, shadow vocabulary, typography role,\ninteraction strength, and tone ramps \u2014 and the component library\nrides along without edit. The component charter (`ship-components`)\ntells authors how to *consume* tokens; this document tells authors\nhow to *build* them.\n\nRead this when you are:\n\n- Adding or editing a theme in `sui/packages/sui/tokens/src/themes.ts`\n  or `themes.css`.\n- Introducing a new token to `sui/packages/sui/tokens/src/tokens.css`.\n- Debugging a \"why doesn't this component flip with the theme\" bug.\n- Helping a user customize Ship past the palette layer.\n\nDo **not** read this to decide which token a component should consume.\nThat's `skills/ship-components/SKILL.md` \u2192 \"Design tokens \u2014 read\nthe highest layer that fits.\"\n\n## The four token layers\n\nIn order of abstraction (highest first). Components read the highest\nlayer that fits; themes override the highest layer they need to.\n\n| Layer | Lives in | Examples | Override to change\u2026 |\n|---|---|---|---|\n| 1. Semantic surfaces & intents | tokens.css | `--surface-0..3`, `--backdrop`, `--hover-bg`, `--selected-bg`, `--divider` | Depth hierarchy, interaction feel |\n| 2. Tone ramps | tokens.css (built from base) | `--tone-success-{fg,bg,bg-solid,border,ring}` for each of default/muted/info/success/warning/destructive/accent/primary | Semantic status color. Override base `--success`/`--warning`/etc. and the ramp rebuilds |\n| 3. Role tokens | tokens.css | `--heading-font`, `--heading-weight`, `--body-font`, `--label-font`, `--ease-out`, `--density-row-py`, `--shadow-hover`, `--shadow-accent` | Typography commitment, motion character, density, semantic shadow intent |\n| 4. Primitives | tokens.css | `--space-1..8`, `--radius-sm..xl`, `--text-xs..4xl`, `--duration-fast`, `--font-display` | Raw scale. Rarely the right override point. |\n\nThe canonical palette (`--background`, `--foreground`, `--card`,\n`--primary`, `--accent`, `--muted`, `--border`, \u2026) sits between layers 1\nand 4. Themes almost always override it. Surfaces fall through to it\nby default (`--surface-1: var(--card)`), so you can change a card color\nonce and lift it everywhere.\n\n## The theme catalog\n\nShip ships seven themes. Every one is a full aesthetic bundle \u2014 palette\n+ surfaces + radii + shadows + motion + typography + interaction.\nComparable in scope to Material or Apple's HIG: a theme swap is a\ndifferent *feel*, not a different hue. There is no `palette`-only\ntier, and the picker is flat.\n\n- **`ship-light`** \u2014 canonical default. Warm off-white, SHIP amber,\n  standard motion, soft shadows, display font for headings.\n- **`ship-dark`** \u2014 sibling. Warm-navy background, warm amber\n  primary. Same motion/radii/typography feel as ship-light.\n- **`paper`** \u2014 editorial reading mode. Cream + terracotta, rounded\n  radii (12\u201320px), slow calm motion (200/300/420ms, `ease-out-soft`),\n  heading font demoted to body, low interaction strength (4\u20137%).\n- **`halo`** \u2014 glassy atmospheric. Pale-blue surfaces with a\n  primary-tinted nested layer, spring motion as the interaction\n  default, snappy 100/150/200ms durations, primary-tinted\n  `--shadow-accent` glow.\n- **`console`** \u2014 90s beige-box nostalgia. Tan/amber + teal, sharp\n  radii (2/3/4/6), inset-style shadows with dark inner edges, mono\n  label font, bold heading weight, chunky tints (12\u201318%).\n- **`contrast`** \u2014 accessibility-hero. Near-black + fused amber\n  primary/accent, pure white foreground, flat 1\u20132px border shadows,\n  blocky tiny radii, bold weights, amped interaction (14/22/30%).\n- **`glass`** \u2014 frosted aero. A soft tinted gradient mesh on the page\n  with translucent `backdrop-filter` surfaces (cards at ~55% white),\n  larger soft radii, an inset top-highlight \"glass edge\" plus a cool\n  drop shadow for lift. Brand orange survives at lower chroma.\n\n**When authoring a new theme, self-audit:** does it bend at minimum\nthe palette + one of (surfaces/motion/radii/shadows) + one of\n(typography/interaction)? If not, it's a recolor pretending to be a\ntheme \u2014 bring it up to the quality bar or don't ship it.\n\n## Authoring a theme\n\nTwo files. Always both.\n\n### 1. Register in `themes.ts`\n\n```ts\n{\n  id: 'midnight-lab',\n  label: 'Midnight Lab',\n  dark: true,               // applies .dark so shadow/elevation flip\n  swatches: [\n    'oklch(0.18 0.02 265)', // background\n    'oklch(0.22 0.02 265)', // card\n    'oklch(0.78 0.14 180)', // primary\n    'oklch(0.68 0.20 320)', // accent\n  ],\n}\n```\n\nThe `ThemeId` union must grow to include the new id. TS will complain\nif you forget.\n\n### 2. Add the CSS block in `themes.css`\n\nGroup the file the way it's already grouped: a comment block with the\ntheme's story, then a `:root[data-theme='\u2026']` rule. Keep the order\n*palette \u2192 surfaces \u2192 shadows \u2192 radii \u2192 motion \u2192 typography \u2192\ninteraction* so diffs between themes are visually comparable.\n\n**Palette skeleton.** Start here \u2014 the palette block every theme needs.\nA real theme keeps going past this (surfaces, radii, motion, shadows,\ntypography, interaction) to earn its slot in the catalog:\n\n```css\n:root[data-theme='midnight-lab'] {\n  --background: oklch(0.18 0.02 265);\n  --foreground: oklch(0.96 0.01 260);\n  --card: oklch(0.22 0.02 265);\n  --card-foreground: oklch(0.96 0.01 260);\n  --popover: oklch(0.22 0.02 265);\n  --popover-foreground: oklch(0.96 0.01 260);\n  --muted: oklch(0.26 0.02 265);\n  --muted-foreground: oklch(0.76 0.02 260);\n  --border: oklch(1 0 0 / 14%);\n  --input: oklch(1 0 0 / 14%);\n  --primary: oklch(0.78 0.14 180);\n  --primary-foreground: oklch(0.15 0.02 265);\n  --accent: oklch(0.68 0.20 320);\n  --accent-foreground: oklch(0.98 0.01 320);\n  --destructive: oklch(0.62 0.22 25);\n  --success: oklch(0.72 0.16 150);\n  --warning: oklch(0.78 0.15 80);\n  --info: oklch(0.72 0.14 220);\n  --ring: oklch(0.78 0.14 180 / 35%);\n}\n```\n\n**Full example** \u2014 same palette, plus the bends that make it a theme:\n\n```css\n:root[data-theme='midnight-lab'] {\n  /* palette \u2014 as above, omitted for brevity */\n\n  /* Surfaces: a warmer nested panel instead of the default mix. */\n  --surface-2: oklch(0.26 0.025 265);\n\n  /* Shadows: cool-tinted, deeper blur for a laboratory-glass feel. */\n  --shadow-sm: 0 1px 3px oklch(0.1 0.04 265 / 50%);\n  --shadow-md: 0 6px 14px -4px oklch(0.1 0.04 265 / 55%);\n  --shadow-lg: 0 14px 30px -6px oklch(0.1 0.04 265 / 60%);\n  --shadow-accent: 0 8px 24px -6px oklch(0.78 0.14 180 / 40%);\n\n  /* Radii: tighter, instrument-panel geometry. */\n  --radius-sm: 3px;\n  --radius-md: 4px;\n  --radius-lg: 6px;\n  --radius-xl: 10px;\n\n  /* Motion: snappy, no spring. */\n  --duration-fast: 100ms;\n  --duration-medium: 150ms;\n  --ease-out: cubic-bezier(0.2, 0.9, 0.3, 1);\n\n  /* Typography: label tracking a touch wider for that schematic feel. */\n  --label-tracking: 0.08em;\n\n  /* Interaction: crisper selected state against deep blue. */\n  --selected-strength: 20%;\n}\n```\n\n### 3. Honour the `.dark` flag\n\nShip's elevation tokens (`--shadow-*`, `--backdrop`) have a dark-mode\npair under `.dark` in `tokens.css`. `applyTheme(id)` toggles that class\nbased on `Theme.dark`. If a theme is inherently dark but `dark: false`,\nyou get light-mode shadows on a dark background and everything looks\nwrong. **Set `dark: true` for any theme whose `--background` is below\n~0.4 L.**\n\nConversely, setting `dark: true` on a light theme will flip shadow\nrecipes you didn't want flipped. The flag is about the *elevation\nstory*, not a mood.\n\n### 4. Override tone-ramp bases, not ramp entries\n\nTone ramps are built in `tokens.css` from `--success`, `--warning`,\n`--destructive`, `--info`, `--accent`, `--primary`. Override those\nbase colors for your theme and the entire ramp rebuilds:\n\n```css\n/* In your theme block \u2014 rebuilds all five --tone-success-* entries. */\n--success: oklch(0.72 0.16 150);\n```\n\nDo **not** write `--tone-success-bg: oklch(...)` in a theme. That\nbypasses the ramp formula and will drift from `-border`, `-ring`,\n`-fg`. The formula is canonical; override the input, not the output.\n\nSpecial case: a theme whose background is so light the default 14% mix\nproduces a washed-out chip may override `--tone-bg-mix` (or the\ninteraction strengths). That's legitimate \u2014 tune the recipe, don't\nbypass it.\n\n## Motion curves and when to pick which\n\n| Curve | Use for |\n|---|---|\n| `--ease-out` | Expressive default. Buttons, panels, most interactions. |\n| `--ease-out-soft` | Surface fades, backdrop appearances, anything that should feel like a room lighting up. |\n| `--ease-spring` | Tactile feedback: press-release on buttons, toggle flips, success confirmations. |\n| `--ease-in` | User dismisses / removes. Rare. |\n| `--ease-in-out` | Scroll-bound or utility motion where symmetry matters. |\n\nA theme can remap: `paper` sets `--ease-out: var(--ease-out-soft)` to\nglobally soften motion character without touching individual\ncomponents. `halo` remaps it to `--ease-spring` for a tactile bounce.\n\nDurations form a scale \u2014 `instant`/`fast`/`medium`/`slow`/`deliberate`.\n`--duration-fast` is the default for hover/focus; `--duration-medium`\nfor layout changes; `--duration-slow`+ for deliberate, premium moments\n(dialog open, toast slide). `prefers-reduced-motion` collapses all of\nthese to 0.01ms automatically.\n\n## Density channel\n\nDensity is an independent axis from theme. Set on `<html>`:\n\n```html\n<html data-density=\"compact\">     <!-- default is \"comfortable\" -->\n<html data-density=\"spacious\">\n```\n\nDensity overrides the `--density-*` channel (`--density-row-py`,\n`--density-row-px`, `--density-row-gap`, `--density-text`,\n`--density-icon`). Every list-shaped component (`ship-list-tile`,\ntable rows, log lines) pulls padding and icon size from this channel\ninstead of the base `--space-*` scale. Result: one attribute on the\nroot element re-proportions every row in the app without a component\nedit.\n\nWhen authoring a new component that presents rows, **read from\n`--density-*`, not `--space-3`.** If the size need is not row-shaped\n(e.g. a button icon), read from the primitives.\n\n## Consumer surfaces \u2014 applying a theme\n\nThe TS module at `sui/packages/sui/tokens/src/themes.ts` is the canonical\nentry point. Three functions:\n\n```ts\nimport { applyTheme, persistTheme, readStoredTheme } from '@ship/tokens/themes'\n\n// On load \u2014 read persisted choice and apply. Safe SSR (returns default).\nconst active = applyTheme(readStoredTheme())\n\n// When user picks a new theme.\napplyTheme('paper')\npersistTheme('paper')\n```\n\n`applyTheme` sets `data-theme` on the root element and toggles the\ncanonical `.dark` class based on `Theme.dark`. It returns the effective\nid (falls back to `DEFAULT_THEME` on invalid input) so you can sync\nstate.\n\nThe canonical UI pattern is the theme picker in the shell: a single\nflat grid of theme cards, swatches visible on each. No tiers.\n\n## Overriding individual tokens\n\nEnd users and integrators don't have to fork a theme to tweak one\nknob. Every Ship token is a CSS custom property at `:root`, so a\ndownstream stylesheet loaded *after* `themes.css` can override any\ntoken:\n\n```css\n/* integrator-overrides.css */\n:root {\n  --radius-lg: 20px;\n  --duration-fast: 220ms;\n}\n:root[data-theme='ship-light'] {\n  --primary: oklch(0.68 0.18 40);\n}\n```\n\nThere's no formal override API yet \u2014 just the cascade. Keep this\nminimal and unsurprising.\n\n## Anti-smells\n\n1. **Never** write `oklch(0 0 0 / 40%)` in a component. Use\n   `var(--backdrop)` (or `--scrim` for a lighter dim).\n2. **Never** write `color-mix(in oklch, var(--success) 14%, transparent)`\n   in a component. Read `var(--tone-success-bg)`. The formula lives in\n   tokens.css once.\n3. **Never** write `rgba(0,0,0,0.08)` or hex anywhere in shipped CSS.\n   Ship is OKLCH-native.\n4. **Never** reach for `var(--card)` when `var(--surface-1)` captures\n   intent. Cards consume the surface layer so a theme that raises card\n   depth (a gradient background, a tinted nested panel) only touches\n   `--surface-1` / `--surface-2`.\n5. **Never** hardcode `\"Inter\"` or `\"Syne\"` in a component. Read the\n   role token (`--heading-font`, `--body-font`, `--label-font`, `--mono-font`).\n   A theme like `paper` demotes display typography by remapping\n   `--heading-font: var(--font-body)` \u2014 components that hardcode the\n   family defeat that.\n6. **Never** ship a `--tone-foo-bg: \u2026` theme override. Override the\n   tone base (`--foo`) and let the ramp rebuild.\n7. **Never** read a theme's `--primary` directly for a status-like\n   intent. Use the tone ramps. `--primary` is a brand slot, not a\n   status signal.\n8. **Never** add `var(--foo, fallback)`. Tokens are guaranteed loaded\n   (tokens.css is imported at the document root). Fallbacks create a\n   parallel, stale system.\n\n## Agent surfaces\n\nSurfaces relevant when working with tokens / themes:\n\n| Tool | When |\n|---|---|\n| `sui/crates/sui/compiler/tokens.json` | The canonical registry of every exposed `var(--\u2026)`. Read it to stay token-locked instead of guessing names; values resolve from `sui/packages/sui/tokens/src/{tokens,themes}.css`. |\n| `sui-compiler build <file>` / `ship sui diagnostics <file>` | After editing a `.sui` template that reads tokens via `var(--\u2026)`, build/diagnose to confirm no broken refs \u2014 the `sui-css-token` lint flags any name not in the registry. |\n| `.deck/catalogs/workspace.json` | Component catalog includes `cssProperties` per primitive \u2014 what CSS custom properties each component accepts as design-token override points. |\n\nFull agent surface inventory: `docs/ship-sui/references/docs/agent-surfaces.md`.\n\n## Relationship to `ship-components`\n\nTwo skills, one boundary:\n\n- **`ship-design-system` (this)** \u2014 authoring tokens and themes.\n  Token names, layer rules, override points, theme catalogue.\n- **`ship-components`** \u2014 authoring `<ship-*>` elements that consume\n  tokens. Component layout, distribution test, shadow DOM conventions,\n  the \"which token for which intent\" table.\n\nIf you're editing `tokens.css`, `themes.css`, `themes.ts`, or a\nconsumer that applies a theme (shell theme picker, marketing site) \u2192\nthis skill. If you're editing\n`packages/components/sui/primitives/src/sui-*/**` \u2192 `ship-components`.\n\n## Open gaps\n\nHonest inventory of places the design system is not yet cohesive,\nsurfaced here so no one re-invents a workaround:\n\n- **Motion is not themable at the curve level everywhere.** `--ease-out`\n  is overridable, but individual components sometimes hardcode a\n  specific cubic-bezier in their `css` templates. Grep for\n  `cubic-bezier` under `packages/components/sui/primitives/src/sui-*/` to\n  find the stragglers; they should all go through the role tokens.\n- **`--tone-bg-mix` / `--tone-border-mix` / `--tone-hover-mix` are\n  legacy.** They're still exposed for `util.toneVars()` but new code\n  reads the ramp variables directly. The legacy names can retire once\n  every consumer is migrated.\n- **Sidebar palette is a parallel mini-system.** `--sidebar-*` tokens\n  predate the surface layer and duplicate what `--surface-*` could\n  express. Kept for Studio compat; a future pass folds them into the\n  surface stack.\n- **`--status-*` chart colors are not tone-ramped.** They expose only\n  the base color; a theme that wants stronger data-viz contrast has to\n  mix in component CSS. A `--status-{name}-{fg,bg,border}` ramp would\n  close this.\n- **Density only affects rows.** A density channel for controls\n  (button height, input height) would let `compact` truly tighten a\n  dashboard. Today `--density-icon` covers icons but not control\n  heights.\n- **No design-system linter.** The anti-smells above are enforced by\n  review, not by a rule. A `stylelint` plugin that flags raw\n  `rgba(`, `oklch(0 0 0 / `, and `cubic-bezier(` inside\n  `packages/components/sui/primitives/src/sui-*/**` would turn this section\n  into a CI check.\n\n## Token schema (planned)\n\nA machine-readable `tokens.json` exporter doesn't ship yet. The\n**canonical source** today is the CSS in\n`sui/packages/sui/tokens/src/tokens.css` itself \u2014 every token\ndeclared at `:root`, every override per `[data-theme='\u2026']`, and the\ndynamic `--tone-*` contract generated by `util.toneVars()` in the\nthemes module.\n\nWhen the exporter lands it will:\n- Emit one entry per token with its `layer`, `intent`, `type`,\n  `default`, and `themable` flag.\n- Drive Figma token sync + IDE autocomplete + design-asset exporters.\n- Power a `bun run lint:tokens` drift checker (fails on hardcoded\n  `rgba`/`oklch`/hex inside component CSS, on `var(--X)` references\n  for unknown tokens, warns on `color-mix(... var(--tone) 14%, \u2026)`\n  formulae that should read the ramp tokens instead).\n\nUntil then, drift is caught visually via `ship-ui-studio` and the\nPlaywright baselines. If you add a token, eyeball every theme in the\ncatalog with the new token in play before committing.\n\nType vocabulary (today, encoded only in the CSS):\n\n| type | example token | notes |\n|---|---|---|\n| `color` | `--primary`, `--tone-success-bg` | OKLCH or an alias. |\n| `radius` | `--radius-lg` | Corner radius. |\n| `space` | `--space-4`, `--density-row-py` | Padding/gap scalar. |\n| `duration` | `--duration-fast` | Motion duration. |\n| `easing` | `--ease-out` | Cubic-bezier role. |\n| `font` | `--heading-font` | Font stack or alias. |\n| `font-weight` | `--font-weight-semibold` | Numeric weight. |\n| `letter-spacing` | `--tracking-wide` | Tracking value. |\n| `shadow` | `--shadow-md`, `--shadow-accent` | Box-shadow composite. |\n| `z-index` | `--z-dialog` | Layer index. |\n| `size` | `--text-lg`, `--density-icon` | Length (px / rem). |\n| `percent` | `--hover-strength` | Percentage value. |\n| `number` | `--leading-normal` | Unitless number. |\n",
});

defineSkill({
  frontmatter: {
    "name": "ship-views",
    "stable-id": "ship-views",
    "description": "USE BEFORE AUTHORING A VIEW. SUI templates compile AOT to ESM and hydrate server-rendered HTML. Pair `.sui` with optional `controller.ts` + `style.css` + required `manifest.jsonc`. Wrong shape and the resolver won't find it.",
    "tags": ["views", "sui", "sdk", "extension-authoring", "ship-elements"],
    "authors": ["ship"],
    "audience": "public",
  },
  assets: [
    { kind: "reference", path: "skills/ship-views/references/form-association.md" },
    { kind: "reference", path: "skills/ship-views/references/shared-context.md" },
    { kind: "reference", path: "skills/ship-views/references/sui-internals.md" },
    { kind: "reference", path: "skills/ship-views/references/template-surface.md" },
    { kind: "scaffold", path: "skills/ship-views/scaffolds/blank/manifest.jsonc" },
    { kind: "scaffold", path: "skills/ship-views/scaffolds/blank/page.sui" },
    { kind: "scaffold", path: "skills/ship-views/scaffolds/dashboard/manifest.jsonc" },
    { kind: "scaffold", path: "skills/ship-views/scaffolds/dashboard/page.sui" },
    { kind: "scaffold", path: "skills/ship-views/scaffolds/form/manifest.jsonc" },
    { kind: "scaffold", path: "skills/ship-views/scaffolds/form/page.sui" },
    { kind: "scaffold", path: "skills/ship-views/scaffolds/list/manifest.jsonc" },
    { kind: "scaffold", path: "skills/ship-views/scaffolds/list/page.sui" },
  ],
  body: "# ship-views\n\nA Ship view is a `.sui` template with an optional paired `.controller.ts`. The compiler emits an ESM per template; the daemon SSRs it; `attach()` wires bindings on the client. Compose `<sui-*>` components inside.\n\n## The rule\n\n**Templates declare. Controllers do.** Dynamic markup goes in the `.sui` template via `{#if}/{#for}/{#match}` and the binding primitives. Controllers own state, refs, lifecycle, async I/O, and the public API the template binds against. A controller that produces markup is a bug \u2014 `sui-controller-purity` rejects it at compile and at edit.\n\nForbidden in `*.controller.ts`:\n- `<el>.innerHTML = \u2026` / `<el>.outerHTML = \u2026` / `<el>.insertAdjacentHTML(\u2026)`\n- A sibling `render.ts` of HTML-string template-literal helpers\n- Re-querying the shadow DOM after `mount` (use `bind:this` for refs)\n- A `.sui` body that is just `<div bind:this={ref}></div>` \u2014 the template is the renderer\n\nFor HTML-blob cases (markdown rendering, third-party SVG generators, icon-pack swaps), use `bindHtml(el, () => string)` from `@ship/sui-runtime`.\n\n## File layout\n\nThe canonical view directory shape:\n\n```\nviews/<name>/                 # first-party view, baked into shipd\n  page.sui                    # route entry \u2014 required\n  controller.ts               # paired controller \u2014 optional\n  style.css                   # page-scoped styles \u2014 optional, daemon auto-links\n  manifest.jsonc              # route metadata \u2014 required\n  dist/                       # generated artifacts (compiler emits here)\n  _components/                # view-private widgets \u2014 optional\n    <view>-<tag>/\n      <view>-<tag>.sui\n      <view>-<tag>.controller.ts\n      dist/\n\n.ship/views/<name>/           # extension-point view, project-local; same shape\n  page.sui\n  controller.ts\n  manifest.jsonc\n  partials/                   # reusable fragments\n```\n\nFive rules:\n\n1. **`page.sui` is the route entry.** The filename stem says what the file IS \u2014 a page, not a reusable widget. Reusable widgets live under `packages/components/sui/primitives/src/sui-*/`.\n2. **`controller.ts`, no stem prefix.** The directory IS the view; pairing is unambiguous. Inside `_components/<tag>/`, the `<tag>.controller.ts` rule from `ship-components` applies.\n3. **`style.css` for page-scoped styles.** Linked automatically. Use `<style>` blocks inside `.sui` for component-scoped styles.\n4. **`manifest.jsonc` is required.** JSONC, not YAML. See \"Manifest\" below.\n5. **`_components/` for view-private widgets.** Tag MUST prefix with the view name (`home-hero` for `views/home/`) so two views can each ship their own `<my-list>` without colliding on `customElements.define`.\n\nPlain `.ts` files (not `controller.ts`, not under `_components/`) are helpers, types, or tests.\n\nEvery first-party view in `views/` uses the `page.sui` + `controller.ts` + `manifest.jsonc` shape. The shell's route mounter (`apps/shell/src/main.ts::mountRoute`) walks only manifest-bearing directories \u2014 a missing `manifest.jsonc` means the view is invisible to the rail, regardless of which template form it ships.\n\n### Worked example: `views/home/`\n\n```\nviews/home/\n  page.sui\n  controller.ts\n  style.css\n  manifest.jsonc\n  _components/\n    home-hero/\n      home-hero.sui\n      home-hero.controller.ts\n```\n\nManifest:\n\n```jsonc\n// views/home/manifest.jsonc\n{\n  \"name\": \"home\",\n  \"path\": \"/\",\n  \"title\": \"Home\",\n  \"layout\": \"default\",\n  \"auth\": \"public\",\n  \"rail\": { \"label\": \"Home\", \"icon\": \"home\" }\n}\n```\n\nPage composes the view-private widget. Reactive state lives in\n`controller.ts`'s `scope:` factory \u2014 there is no `<state>` directive:\n\n```html\n<!-- views/home/page.sui -->\n<div class=\"home\">\n  <home-hero prop:title={'Welcome to Ship'}></home-hero>\n  <button on:click={() => seen.set(seen() + 1)}>Bump ({seen()})</button>\n</div>\n```\n\n```ts\n// views/home/controller.ts\nimport { controller, signal } from '@ship/sui-runtime'\nimport type { Scope } from './page.sui'\n\nexport default controller<Scope>({\n  scope: () => {\n    const seen = signal(0)\n    return { seen }\n  },\n})\n```\n\nScaffold a new view:\n\n```sh\ndeck scaffold view settings\ndeck scaffold view settings --component settings-toolbar\n```\n\n## Template surface (compact)\n\n```html\n<!-- live data subscriptions (server-driven, SSE-backed) -->\n<data name=\"agents\" source=\"ship.agents.summaries()\" live />\n<data name=\"settings\" source=\"ship.settings.read()\" />\n\n<!-- reactive args \u2014 re-subscribes when selectedId changes.\n     `selectedId` is a signal declared in the controller's `scope:`\n     factory (see Controller contract below). -->\n<data name=\"agent\" source=\"ship.agents.detail({id: $scope.selectedId()})\" live />\n\n<!-- props (component .sui files only) -->\n<prop name=\"tone\" values=\"default|primary\" default=\"default\" reflect />\n<prop name=\"disabled\" type=\"bool\" default=\"false\" reflect />\n\n<!-- mutations (write-side complement to <data>) -->\n<form mutation=\"ship.workspaces.activate\">\n  <input name=\"id\" />\n  <button type=\"submit\" disabled={$mutation().kind === 'loading'}>Switch</button>\n</form>\n\n<!-- control flow \u2014 block syntax -->\n{#if loading()}\u2026{:else if error()}\u2026{:else}\u2026{/if}\n{#for a of items() key={a.id}}\u2026{/for}\n{#match status()}{:case 'idle'}\u2026{:default}\u2026{/match}\n{#while-loading of agents}<p>Loading\u2026</p>{/while-loading}\n{#while-error err of agents}<p>{err.message}</p>{/while-error}\n\n<!-- bindings -->\n<button on:click={() => count.set(count() + 1)}>+</button>\n<input bind:value={query} />\n<input type=\"checkbox\" bind:checked={enabled} />\n<div class:active={isActive()} style:color={color()} attr:href={url()}>\n  {count() === 1 ? '1 item' : count() + ' items'}\n</div>\n\n<!-- pass complex objects (signals, arrays, structured data) to a child component -->\n<sui-grid prop:rows={rows} prop:columns={columns()} prop:total={count()}></sui-grid>\n\n<!-- template hooks (browser-platform bridges) -->\n<use-kbd><on key=\"/\" action=\"search\" /></use-kbd>\n<use-click-outside bind:target={dialogEl} bind:action={close} />\n<use-focus-trap bind:target={dialogEl} />\n<use-clipboard />\n<use-title source={title} />\n<use-scroll-anchor />\n<use-intersection callback={onIntersect} />\n\n<!-- slots (components only) -->\n<slot name=\"leading\"></slot>\n<slot></slot>\n```\n\nThe file IS the view \u2014 no `<component>` wrapper. Top-level directives (`<data>`, `<prop>`, `<use-*>`) declare scope; the rest is the body. Reactive component state is declared in the paired controller's `scope:` factory, not the template.\n\nFor full examples see `references/template-surface.md`.\n\n## Controller contract\n\n```ts\n// view.controller.ts\nimport { controller, signal, derived, shared, useShared, useFilteredList, unwrap } from '@ship/sui-runtime'\nimport type { Scope } from './view.sui'   // compiler-emitted .d.ts\n\nexport default controller<Scope>({\n  scope: ({ agents }) => {\n    const filter = signal('')\n    const total  = derived(() => unwrap(agents)?.items.length ?? 0)\n    const list   = useFilteredList(agents, (a, q) => a.name.includes(q))\n    return { filter, total, list }\n  },\n  on: {\n    newAgent: ({ navigate }) => navigate('/agents/create'),\n  },\n  mount:   ({ state }, el) => { /* runs after first DOM attach */ },\n  unmount: ({ state })     => { /* runs on dispose */ },\n})\n```\n\nThree blocks, each one job:\n\n- **`scope:`** \u2014 factory that contributes typed signals, derived, hooks, shared providers and consumers. Runs first; entries land in template scope. Replaces the historical state/derived/use split.\n- **`on:`** \u2014 multi-statement event handlers. The first arg is a writable proxy: `s.count = s.count + 1` mutates state.\n- **`mount:` / `unmount:`** \u2014 lifecycle. Runs after first attach / on disconnect.\n\n## Cross-component reactive context\n\nFor complex components that split into a public outer + internal inner pieces, share signals from parent to descendants without prop-drilling. The parent calls `shared(signal(\u2026))` inside its `scope:`; descendants call `useShared<ParentScope>('parent-tag')` to resolve the same signal identity. `useShared` walks DOM ancestry across shadow boundaries; returns `null` when no provider is found.\n\nTwo controller tiers \u2014 smallest that fits: **no controller** (pure `.sui`, single-expression inline handlers only) or **paired controller** (state, derived, hooks, lifecycle, multi-statement handlers). A multi-statement `on:event` in a tier-1 file is a compile error.\n\nFull contract, worked examples, lookup semantics: `references/shared-context.md`.\n\n## Form-aware components\n\nComponent `.sui` files only \u2014 views don't participate in form submission. When a `<sui-*>` component is itself a form control, mark its value prop with `form-value` and (optionally) its disabled prop with `form-disabled`:\n\n```html\n<prop name=\"value\" type=\"string\" default=\"\" form-value />\n<prop name=\"disabled\" type=\"bool\" default=\"false\" reflect form-disabled />\n```\n\nThe compiler emits `static formAssociated = true` and the runtime auto-wires `setFormValue`, `formResetCallback`, `formStateRestoreCallback`, and `formDisabledCallback`. Drop any hand-rolled `ElementInternals` plumbing \u2014 the runtime owns it. The setup ctx still surfaces `internals?: ElementInternals` for `setValidity()` / `checkValidity()` paths.\n\nWorked examples: `packages/components/sui/primitives/src/sui-input/sui-input.sui`, `packages/components/sui/primitives/src/sui-select/sui-select.sui`. Full contract and constraints: `references/form-association.md`.\n\n## Live data envelope\n\n```ts\ntype LiveState<T> = { kind: 'loading' } | { kind: 'error'; error: Error } | { kind: 'ok'; value: T }\n\nunwrap(sig)               // T | undefined\nunwrapOr(sig, fallback)   // T\n```\n\n```html\n{#while-loading of agents}<p>Loading\u2026</p>{/while-loading}\n{#while-error err of agents}<p>{err.message}</p>{/while-error}\n{#if agents().kind === 'ok'}\n  {#for a of agents().value.items key={a.id}}\u2026{/for}\n{/if}\n```\n\n## Live SSE data\n\nThe full end-to-end loop for `<data live>` \u2014 what's on the wire, where each\npiece lives, and how to confirm it locally.\n\n### Wire path\n\n```\n.sui  <data live source=\"ship.git.overview()\" live />\n  \u2193 sui-compiler\n.js   $scope.git = live(() => ship.git.overview())\n  \u2193 @ship/sdk-client createClient()\nHTTP  GET /_deck/data?expr=ship.git.overview()    [text/event-stream]\n  \u2193 apps/shipd/src/views/sui_data.rs\nRust  registry::call(\"git.overview\", {}) \u2192 initial; subscribe `config` ns\n  \u2193 kernel event \u2192 re-resolve\nSSE   event: patch \\n data: {\"op\":\"set\",\"value\":{\u2026}}\n  \u2193 makeSseSource() in @ship/sdk-client\nsig   LiveSignal<T> flips loading \u2192 ok \u2192 ok (per patch)\n  \u2193 bindWhileLoading / bindWhileError / bindIf\nDOM   morphs in place \u2014 no re-render of the surrounding view\n```\n\nThe compiler turns each `source=` attribute into a literal expression\ninside `live(() => \u2026)`. The expression must resolve to a `LiveSource<T>`\n(has `.subscribe(sig)`) \u2014 that's the shape every SDK method returns, so\n`ship.<domain>.<op>(args)` is the idiomatic form. Don't hand-roll an\n`EventSource` in the controller; let the SDK own the wire.\n\n### Worked example: `views/home/`\n\nThe home view ships the canonical demo. `<data live>` subscribes to\n`ship.git.overview()`; the daemon's git_signal singleton refreshes on\nfilesystem events under `.ship/` and on git ops, so staging a file in\nthe worktree visibly flips the \"clean / N changed\" badge without a\npage reload.\n\n```html\n<!-- views/home/page.sui -->\n<data name=\"git\" source=\"ship.git.overview()\" live />\n\n<section class=\"home-live\" aria-live=\"polite\">\n  {#while-loading of git}\n    <p>Connecting\u2026</p>\n  {/while-loading}\n  {#while-error err of git}\n    <p>SSE error: {err.message}</p>\n  {/while-error}\n  {#if git().kind === 'ok'}\n    <p>\n      Branch <strong>{branch()}</strong>\n      \u00b7 <span>{headShort()}</span>\n      \u00b7 <span>{clean() ? 'clean' : changedSummary()}</span>\n    </p>\n  {/if}\n</section>\n```\n\n```ts\n// views/home/controller.ts\nimport { controller, signal, unwrapOr } from '@ship/sui-runtime'\nimport type { GitOverview } from '@ship/sdk-client'\nimport type { Scope } from './page.sui'\n\nconst EMPTY: GitOverview = { branch: '', head_short: '', clean: true,\n  total_changed: 0, staged: [], modified: [], untracked: [],\n  commits: [], branches: [], ahead: 0, behind: 0, workspace_id: null }\n\nconst read = ({ git }: Scope) => unwrapOr(git, EMPTY)\n\nexport default controller<Scope>({\n  scope: () => ({ seen: signal(0) }),\n  derived: {\n    branch:    (s) => read(s).branch,\n    headShort: (s) => read(s).head_short,\n    clean:     (s) => read(s).clean,\n    changedSummary: (s) => `${read(s).total_changed} changed`,\n  },\n})\n```\n\n### Verify locally\n\n```bash\njust dev-restart            # rebuilds shipd + restarts the daemon\nPORT=$(cat $(./target/debug/ship daemon dir)/port)\n\n# 1. Confirm the SDK channel is live (Ctrl-C after a few `patch` events).\ncurl -N \"http://localhost:$PORT/_deck/data?expr=$(printf 'ship.git.overview()' | jq -sRr @uri)\"\n\n# 2. Open the view, stage a file, watch the badge flip.\nxdg-open \"http://localhost:$PORT/\"            # or open in your browser\ntouch CHANGE_ME && git add CHANGE_ME           # status flips to \"1 changed\"\ngit restore --staged CHANGE_ME && rm CHANGE_ME # back to \"clean\"\n```\n\n### When to add a new live source\n\nA new `ship.<domain>.<op>(...)` live source needs three things:\n\n1. **Registry entry** \u2014 `apps/shipd/src/services/registry/<domain>.rs` returns\n   the current value synchronously. The dispatcher in `registry.rs` and the\n   schema row in `services/schema.rs` keep the catalog honest.\n2. **SDK method** \u2014 `packages/sdk-client/src/index.ts` exposes the\n   method returning `makeSseSource(...)` against `/_deck/data?expr=\u2026`.\n3. **(Optional) signal-backed source** \u2014 if the data has a tighter wake-up\n   surface than the default `config` kernel namespace, mirror the pattern in\n   `apps/shipd/src/views/git_signal.rs`: a `Signal<Value>` plus a singleton\n   background task that updates it. `sui_sse.rs::is_signal_backed_expr` picks\n   up the new registry name automatically.\n\nThe view-side surface (`<data live source=\"ship.X.Y()\" live />` + controller\nderived helpers) is the same regardless of which mechanism powers the\nre-resolves on the daemon.\n\n## Manifest\n\n```jsonc\n// views/<name>/manifest.jsonc \u2014 JSONC, the house format\n{\n  \"name\": \"my-view\",        // stable identity; matches directory\n  \"path\": \"/my-view\",       // URL pattern \u2014 `/path` or `/path/[param]`\n  \"title\": \"My View\",       // <title> when mounted\n  \"layout\": \"default\",      // layout slot key\n  \"auth\": \"public\",         // \"public\" | \"user\" | \"admin\"\n  \"rail\": {                 // optional \u2014 shell rail entry\n    \"label\": \"My View\",\n    \"icon\": \"sparkles\"\n  },\n  \"preload\": [              // optional \u2014 SDK calls warmed before mount\n    \"ship.workspaces.list()\"\n  ],\n  \"events\": [               // optional \u2014 SSE channels to subscribe to\n    \"config.changed\"\n  ]\n}\n```\n\nThe shell's route mounter (`apps/shell/src/main.ts::mountRoute`) walks `views/` at build/start time and registers each `manifest.jsonc`. URL \u2192 view module resolution and `[param]` capture happen there. For daemon route shapes, the SSR + attach pipeline, and the marker contract, see `references/manifests-routes.md`.\n\n## Hard rules\n\n1. **Mutations in `on.*` handlers only.** `s.x = v` in a derived or template expression is a type error.\n2. **Subscriptions in `<data>` elements.** Never call `ship.*` outside `<data>`.\n3. **Templates declare; controllers do.** Declarative \u2192 template; imperative \u2192 controller.\n4. **No `<script>` blocks in templates.** Logic in paired `.controller.ts`.\n5. **No `customElements.define` in view code.**\n6. **Read signals by call:** `count()` reads, `count.set(n)` writes.\n7. **Block syntax for control flow.** `{#if}/{#for}/{#match}/{#while-loading}/{#while-error}`.\n8. **No `el.innerHTML/.outerHTML/.insertAdjacentHTML`** in `*.controller.ts`. Use `bindHtml()` for HTML-blob cases.\n9. **`{#for}` blocks declare `key={EXPR}`.** Without a stable key the reconciler falls back to positional matching and DOM identity (focus, selection, in-place edits) is lost on reorder. `sui-for-key` lint warns when missing.\n10. **`on:<event>` event names must match the platform whitelist:** DOM events (`click`, `input`, \u2026), `sui-*` / `ship-*` custom-event families, and `tag:event`-prefixed cross-component events. `sui-event-name` lint warns on typos like `on:Click` with Levenshtein suggestions.\n11. **Form-aware components use `form-value` / `form-disabled` on `<prop>`** \u2014 never hand-roll `attachInternals()` / `setFormValue` / `formResetCallback`. See \"Form-aware components\" above.\n\n## Performance gates\n\nThree recipes guard runtime + visual surface before a PR:\n\n- `just bench` \u2014 capture a fresh js-framework-benchmark baseline; commit the\n  resulting `tests/perf/baselines/<date>-baseline.json`.\n- `just bench-check` \u2014 re-runs the bench suite against the current bundle and\n  fails if any op regresses >15% vs the latest committed baseline. Override\n  the threshold via `BENCH_REGRESSION_THRESHOLD`. Run before merging any\n  runtime-affecting change.\n- `just visual` \u2014 Playwright screenshots vs committed PNGs (daemon must be\n  running). Pass `--update-snapshots` to refresh baselines after a\n  deliberate visual change.\n- `just smoke-deploy` \u2014 full build + daemon restart + every baselined view\n  curled + Playwright visual suite. Proxy for what CI will see; aim for\n  green before opening a PR.\n\n## Agent surfaces (MCP)\n\nEvery agent-facing tool relevant to view authoring. Full catalog with\ngroupings + the LSP CLI + `@ship/sui-eval` scoring lives in\n`docs/ship-sui/references/docs/agent-surfaces.md`.\n\n### Scaffold + diagnose loop\n\n| Tool | When |\n|---|---|\n| `ship ops call create.surface --json '{\"kind\":\"view\",\"name\":\"<name>\"}'` | Emit a fresh `views/<name>/` skeleton. |\n| `ship sui diagnostics <file>` | First thing after writing. Returns `[{ line, col, severity, message, source }]`. Same analyser as the VS Code LSP. |\n| `ship sui hover <file> <line> <col>` | Type / directive / prop info at a cursor. Use when you're not sure what shape a binding wants. |\n| `ship sui dependencies <file>` | What tags / signals / routes this file reaches into. Read before edits to scope blast radius. |\n| `sui-compiler build <file>` | Full lowering pipeline. Catches issues `ship sui diagnostics` (parse + analyse only) misses. |\n\n### Source I/O\n\n| Tool | When |\n|---|---|\n| Read tool on `views/<name>/` | Read one file or the full view dir. |\n| Edit/Write tools on `views/<name>/` | Write \u2014 triggers recompile + SSE notify. |\n\n### Live mount + poke\n\n| Tool | When |\n|---|---|\n| `ship ops call view.patch_session --json '{\"selector\":\"...\",\"html\":\"...\"}'` | Morph an HTML subtree in place. |\n| `ship ops call view.signal_session --json '{\"name\":\"...\",\"value\":...}'` | Set a reactive signal as if the user did via UI. |\n\n### Theme + collections\n\n| Tool | When |\n|---|---|\n| `sui/crates/sui/compiler/tokens.json` | The active theme's token registry. Read before authoring CSS so you stay token-locked. |\n| `ship ops call collections.list` | Schemas for project-defined `<data source=\"\u2026\">` collections. Use before writing a `<data>`. |\n| `ship ops call annotation.post --json '{\"...\"}'` | Pin a TODO/comment to a DOM node; surfaces in the dev panel. |\n\n### Verify before \"done\"\n\n**The gate is the compiler, and it is always available. Run it.** Do not skip\nthis because an MCP tool is missing, and never substitute `tsc` \u2014 Ship does not\nbundle TypeScript. Controller checking is the oxc-based lint pass *inside* the\ncompiler (`sui/crates/sui/lint`), so the compiler is the one source of truth for\nboth template and controller correctness.\n\n**Gate 0 \u2014 `sui-compiler build` (zero setup, no MCP, no daemon).** The compiler\nships on PATH (`~/.cargo/bin/sui-compiler`, installed by `just install-dev`).\nIt runs grammar + semantic checks **and the full lint pass** (the same rules the\nLSP and `view_lint` surface, e.g. `sui-css-content-text`, `sui-controller-purity`).\nA non-zero exit / printed `Error:` is a hard fail \u2014 fix and re-run until clean:\n\n```sh\nsui-compiler build views/tasks/page.sui     # one file\nsui-compiler build views/tasks/             # whole view dir (template + siblings)\n```\n\nThis is the gate you can *always* run \u2014 in any session, with or without the\nShip MCP server connected. If you only do one thing before declaring done, do this.\n\n> **The MCP tool surface has been removed.** Use `sui-compiler build` as the\n> durable gate. `@ship/sui-eval` (`runEval({ description, spec, catalog })`) remains\n> the behavioural-scoring path when you authored against an interaction-script + asserts\n> contract.\n\n```sh\n# The verify loop:\n# 1. write template + controller\n# 2. sui-compiler build views/tasks/   # the gate \u2014 fix every error, repeat until clean\n# 3. web-qa skill                       # render + console + theme evidence\n```\n\n## LSP integration\n\nThe same analyser that backs `sui_diagnostics` is exposed as a standalone\nlanguage server.\n\n| Surface | How |\n|---|---|\n| **CLI** (agents) | `ship sui diagnostics` / `hover` / `dependencies`. See above. |\n| **VS Code** | `apps/vscode/` auto-launches the `sui-lsp` binary on `.sui` open. |\n| **Any editor / Claude Code / opencode** | Point the LSP client at the `sui-lsp` binary (`sui/crates/sui/lsp/`, on PATH via `just install-dev`) over stdio. Claude Code uses the repo `.lsp.json`. |\n| **Browser** | `@ship/sui-lsp-browser` runs the WASM `analyze` (same Rust grammar) for diagnostics in the docs playground; hover/completion are static-dictionary for now. |\n\n## Components\n\nViews compose `<sui-*>` components from `packages/components/sui/primitives/`. See the `ship-components` skill for the component contract.\n\n## Error handling\n\nWrap fault-prone subtrees in `<sui-error-boundary>`. Throws inside the\nwrapped subtree (effects, controller mount, render expressions) are caught\nand the fallback is rendered. Use `fallback-text` for a string fallback or\n`<template slot=\"fallback\">` for custom markup. Call `.reset()` on the host\nelement to retry.\n\n```html\n<sui-error-boundary fallback-text=\"Couldn't load feed\">\n  <sui-some-component prop={maybeThrows()}></sui-some-component>\n</sui-error-boundary>\n```\n\nThe boundary fires `sui-error-boundary:caught` (bubbling, composed) with\n`{ error, source }` where `source` is `'effect' | 'mount' | 'render' |\n'unknown'`. Nest boundaries to scope recovery \u2014 the innermost catches\nfirst, mirroring React.\n\n## Portal\n\nUse `<sui-portal target=\"body\">` to teleport its slotted content into a\ndifferent DOM location (typically `document.body`) for stacking-context\nescape \u2014 modals, dropdowns, popovers, toasts. The portal element stays\nthe logical parent, so reactive bindings inside continue to fire and\nevents bubble through the original DOM ancestry, not the target's.\n\n```html\n<sui-popover-trigger>Open</sui-popover-trigger>\n<sui-portal target=\"body\">\n  <sui-popover-panel>Floating content</sui-popover-panel>\n</sui-portal>\n```\n\n`target` is a CSS selector resolved against `document`; the default is\n`body`. Common alternatives: `#modal-root`, `[data-portal-root]`. A\nmissing target warns to the console and leaves children in place\n(graceful degradation). Multiple portals targeting the same selector\nappend in registration order \u2014 z-index ordering remains the author's\njob. Teleported nodes are removed from the target when the portal\nitself unmounts.\n\n## Transitions\n\nWrap elements with `transition:<name>` to animate enter/exit on `{#if}`\nor `{#for}` mount/unmount. Built-ins: `fade`, `slide`, `scale`. Use\n`in:<name>` + `out:<name>` for asymmetric directions. Custom transitions\nexport `{ in?, out? }` matching the `Transition` type.\n\n```html\n{#if visible()}\n  <div transition:fade={{ duration: 200 }}>...</div>\n{/if}\n\n<!-- Asymmetric: slide in, fade out -->\n{#if visible()}\n  <div in:slide out:fade={{ duration: 150 }}>...</div>\n{/if}\n```\n\nThe compiler auto-imports built-in names from `@ship/sui-runtime`;\ncustom transitions must be on the controller's `scope:` factory.\nOut-transitions defer DOM removal until the animation finishes; a\nrapid toggle while an out-transition is in flight rescues the same\nnodes so the in-transition picks up from current state without\nre-rendering. `prefers-reduced-motion: reduce` collapses duration and\ndelay to `0` automatically. Transitions don't surface in SSR output \u2014\nthey register on element construction (CSR + hydrate paths).\n\n## When components use these \u2014 real-world exemplars\n\nThe portal + transition primitives are composed by three platform\ncomponents. Read them when you're authoring a new overlay-style\nprimitive:\n\n| Primitive | Portal role | Transition |\n|---|---|---|\n| `sui-dialog` | `<sui-portal target=\"body\">` teleports a `[data-sui-dialog-open]` marker so page-level styles can react to \"any modal open\" via `:has` selectors. The dialog element itself stays in shadow because native `showModal()` already escapes parent stacking context via top layer. | `transition:fade={{ duration: 150 }}` on the #if-mounted content wrapper drives open/close polish. |\n| `sui-popover` | Same pattern \u2014 `[data-sui-popover-open]` marker portaled to body. Native `popover=\"auto\"` handles top-layer + light dismiss; the marker is for page-level hooks. | `transition:scale={{ duration: 120 }}` on the #if-mounted `.surface`. |\n| `sui-toast-container` | `<sui-portal target=\"body\">` teleports the actual stack of toasts. Toast containers *genuinely* need to escape ancestor `transform` / `overflow` containing blocks \u2014 `position: fixed` alone isn't enough. The slot stays in shadow; the controller adopts slotted + `show()`-created toasts into the body-side stack. | Each `<sui-toast>` uses asymmetric `in:slide={{ axis: 'y' }}` + `out:fade` on its `#if visible()` panel. The controller flips `visible` to false on `close()` and tears the host down after the out-transition. |\n\nThe takeaway: where the native platform already provides top-layer\nrendering (`<dialog>.showModal()`, `popover=\"auto\"`), `<sui-portal>`\nis best used for *document-level state markers* that page chrome can\ntarget \u2014 not for re-portaling the panel itself, which would break\nslot projection (slots only project inside their host's shadow tree;\nonce teleported they detach). Where the native platform does NOT (a\nfixed-position toast stack inside a transformed ancestor),\n`<sui-portal>` is the right tool to teleport the whole thing.\n\n## Rail composition\n\n`<sui-rail>` is the left chrome container; views are addressed via\nslotted children. Author the rail in the shell HTML once and let the\ncontroller handle features (project switcher, session-count badge,\nlauncher / create, active-view tracking, click-outside dismiss).\n\n```html\n<!-- apps/shell/index.html \u2014 single static instance -->\n<sui-rail id=\"ship-rail\">\n  <sui-rail-item icon=\"layout-dashboard\" label=\"Dashboard\" href=\"/\" data-view=\"dashboard\"></sui-rail-item>\n  <sui-rail-item icon=\"bot\" label=\"Agents\" href=\"/agents\" data-view=\"agents\"></sui-rail-item>\n  <sui-rail-item icon=\"git-branch\" label=\"Git\" href=\"/git\" data-view=\"git\"></sui-rail-item>\n  <sui-rail-item icon=\"sparkles\" label=\"UI Studio\" href=\"/ship-ui-studio\" data-view=\"ship-ui-studio\"></sui-rail-item>\n</sui-rail>\n```\n\nThree composition rules:\n\n1. **`data-view` is the active-view key.** The rail's controller\n   listens for `sui:navigate` (and `popstate`) and reads\n   `location.pathname.split('/').filter(Boolean)[0]`. The matching\n   `<sui-rail-item[data-view]>` child gets the `active` attribute set;\n   all others have it cleared. No manual wiring in the shell.\n2. **Mission Control is baked in.** `<sui-rail>` renders its own\n   `<sui-rail-item part=\"mission-control\" data-view=\"mesh\">` inside\n   the nav after the slot. The controller polls `/api/v1/pty-sessions`\n   every 15s and reflects the live count to the item's `badge`\n   attribute (suppressed at 0). Don't re-slot a Mission Control item\n   from the shell \u2014 it would collide.\n3. **Footer features dispatch bubbled events.** Project badge,\n   launcher, create \u2014 every footer action emits a composed bubbling\n   event the shell can listen for at the document level:\n   `sui-rail:toggle`, `ship:launcher:open`, `ship:create:open`,\n   `ship:project:change`. The project popover is wrapped in\n   `<sui-portal target=\"body\">` with `transition:fade={{ duration: 150 }}`\n   so it escapes the rail's `overflow: hidden` + stacking context\n   cleanly.\n\nThe rail controller fetches `/api/v1/projects` for the switcher and\npositions the portaled popover against the project-badge bounding\nrect (updates on `resize` / capturing `scroll`). When you need the\nselected project, listen on `document` for `ship:project:change`.\n\n## Morph (in-place DOM patching)\n\nAdd the `morph` directive to any element whose contents are produced\nby `bindRawHtml` (or another HTML-string-replacing path) to preserve\nfocus, scroll, selection, and event-listener identity across\nre-renders:\n\n```html\n<div morph>\n  {/* server-rendered HTML \u2014 bindRawHtml in the controller */}\n</div>\n```\n\nWithout the directive, `bindRawHtml` replaces `innerHTML` and destroys\nany in-progress UI state on the children. With the directive,\n`bindRawHtml` routes updates through `morph()` \u2014 same-id elements are\nreused (even across reorders), attributes are swapped only when they\nchange, and only truly-different subtrees are touched.\n\n`bindHtml` always morphs (no directive required); the directive only\nmatters for `bindRawHtml` and any future HTML-producing binding.\n\nLimitations: doesn't preserve `<select>` value across morphs (browser\nquirk), doesn't morph into shadow DOM, matches only by `id` and\ntag-plus-position (no content-similarity heuristics).\n\n## References\n\n| When you need | Read |\n|---|---|\n| Worked template examples, controller tiers, hooks, `<prop>` attribute reference, manifest schema, daemon routes, SSR + attach pipeline | `references/template-surface.md` |\n| Form-aware components \u2014 full contract, constraints, controller access to `internals` | `references/form-association.md` |\n| Cross-component reactive context (`shared` / `useShared`), controller tiers | `references/shared-context.md` |\n| Working on the SUI codebase itself (compiler, runtime, daemon SUI handlers) | `references/sui-internals.md` |\n\n## Hot reload\n\nIn dev mode, `.sui` and `.controller.ts` edits trigger an in-place view\nre-mount without a full page reload. shell's boot calls\n`initHmr()` (apps/shell/src/hmr.ts), which uses\n`subscribeHmr` from `@ship/sui-runtime/dev` to listen on shipd's\n`/_deck/dev/hmr` SSE channel. When the Bun bundler emits\n`module-changed` for `views/<name>/view` and `<name>` matches the\ncurrent route, shell cache-busts the rebuilt `view.js` and re-mounts\nthrough the existing `mount()` lifecycle.\n\nOther bundles (sui-components, sui-components/*, sui-runtime, sui-dom,\nship-elements) trigger a full reload \u2014 custom-element re-registration\nwithout reload isn't yet supported. Compile errors are logged to the\nbrowser console with `file:line`. Production builds inline\n`import.meta.env.DEV` to `false`, so HMR is a no-op and no\nEventSource opens.\n\nRun `just dev` and edit any view file under `views/<name>/` to see it\napply in place.\n\n## Routes\n\nURL state is reactive. Import from `@ship/sui-runtime/route`:\n\n```ts\nimport { controller } from '@ship/sui-runtime'\nimport { route, defineRoutes, navigate } from '@ship/sui-runtime/route'\nimport type { Scope } from './page.sui'\n\ndefineRoutes([{ pattern: '/agents/:id' }])\n\nexport default controller<Scope>({\n  scope: () => {\n    const r = route()\n    return {\n      agentId: r.params.id,             // Signal<string | undefined>\n      tab:     r.query.tab,             // Signal<string | undefined>\n      goHome:  () => navigate('/'),\n    }\n  },\n})\n```\n\n`route().params.<name>` and `route().query.<name>` are signals \u2014 call them\nto read (and subscribe) inside `effect`, `derived`, or a `<data>` source\nthunk. Param keys are lazy: reading `r.params.id` the first time creates\nthe signal and seeds it from the current pathname. `defineRoutes([...])`\nregisters patterns whose `:param` segments populate `params`; safe to\ncall any number of times. Mutate the URL via `navigate(path)` \u2014\nassigning to `params`/`query` throws. `popstate` and `navigate` both\nfire the global updater so every attached `route()` instance re-evaluates.\n\n## Controller scope is typed\n\nThe compiler emits `Scope` in `<file>.sui.d.ts` covering BOTH template\nreactive identifiers AND the keys you add via `scope: () => ({...})`\nfactory. `mount(state, host)` sees direct, typed members:\n\n```ts\nmount(state, host) {\n  state.caughtError.set(null)  // Signal<Error | null>\n  state.reset()                 // () => void\n}\n```\n\nNo casts. No \"scope signals missing\" defensive throws. If a key is\nmistyped or missing, the TypeScript compiler catches it.\n",
});

defineSkill({
  frontmatter: {
    "name": "spawn-agent",
    "audience": "internal",
    "stable-id": "spawn-agent",
    "description": "Dispatch a specialist agent through Helm \u2014 helmd compiles, provisions the worktree, and launches the harness. `dispatch-portable.sh` is the thin CLI client; it boots helmd locally if the daemon is down.",
    "tags": ["commander", "dispatch", "worktree"],
    "authors": ["ship"],
  },
  assets: [
    { kind: "script", path: "skills/spawn-agent/scripts/_worktree-base.sh" },
    { kind: "script", path: "skills/spawn-agent/scripts/dispatch-portable.sh" },
  ],
  body: "# Dispatch\n\nDispatch a specialist agent to a job. **helmd owns dispatch** \u2014 it compiles the agent, provisions the worktree + branch, and launches the harness into a tmux tab you can attach to. Commander, the Launch UI, and `dispatch-portable.sh` are all clients of it.\n\n## Hard rule for dispatched agents\n\n**NEVER push to a remote.** Agents commit locally to their branch. The orchestrator gates by reading the local worktree state. `git push` is a side-effect on a shared remote that requires explicit human authorization \u2014 never assume it. If a job spec ever says \"push\", treat it as a typo and stop at commit.\n\nhelmd installs a shared pre-push guard at `<main-repo>/.git/hooks/pre-push`\nduring worktree provisioning: it blocks any push of `refs/heads/job/*` and\ncan only be bypassed with `SHIP_ALLOW_PUSH=1 git push ...` (deliberate,\none-shot). Agents running with bypassPermissions cannot silently push.\n\n## How to dispatch\n\n**Primary \u2014 through Helm.** Call the `dispatch_job` MCP tool (or the Launch UI, or `dispatch-portable.sh` \u2014 all POST helmd). helmd compiles the agent, provisions the worktree, and launches the harness. You never create worktrees or compile configs yourself.\n\n`dispatch-portable.sh` boots helmd locally if it isn't already running\n(looks for `target/debug/helmd` relative to the repo root, else `helmd` on\nPATH), so the same command works from a fresh shell or after a daemon bounce.\n\nThe job params (MCP tool fields, or CLI flags on `dispatch-portable.sh`):\n\n| Flag | What |\n|------|------|\n| `--slug <name>` | Worktree dir and branch name (required) |\n| `--agent <agent>` | Ship agent profile (required) |\n| `--spec <file>` | Path to job spec (required) |\n| `--base <branch>` | Advisory only \u2014 helmd applies its own `job/<slug>` convention. Accepted for back-compat. |\n| `--harness <cli>` | Override the harness binary: `claude` (default), `omp`, `codex`, etc. The harness must be authenticated in the launching terminal's environment. |\n| `--model <id>` | Override model for the spawned agent |\n| `--dir <path>` | Worktree base dir (informational; helmd resolves its own) |\n| `--dry-run` | Resolve + print the plan; do NOT POST helmd / spawn |\n\nFlags accepted for back-compat with the old in-script spawn (no-op on the\nRPC path): `--no-open`, `--skill <id>`, `--confirm`, `--batch <name>`,\n`--no-bootstrap`, `--ship-bootstrap`. They're silently swallowed so existing\ncallers don't break \u2014 don't add new code that depends on them.\n### Environment variables\n\nThese are personal configuration \u2014 set in your shell profile, not in project files.\n\n| Variable | What |\n|----------|------|\n| `SHIP_DEFAULT_TERMINAL` | Force terminal: `tmux`, `wt`, `iterm`, `vscode`, `warp`, `manual` |\n| `SHIP_WORKTREE_DIR` | Base directory for worktrees (default: `~/dev/Ship/worktrees`) |\n| `SHIP_GLOBAL_DIR` | Ship global data dir (default: `~/.ship`) |\n| `SHIP_AGENT_MODEL` | Default model for spawned agents |\n| `SHIP_DISPATCH_CONFIRM` | Set to `1` to always prompt before dispatch |\n| `SHIP_PROVIDER_CLI` | Override provider binary: `claude`, `codex`, `gemini`, `opencode` |\n| `WT_PROFILE` | Windows Terminal profile name to launch panes under (default: `$WSL_DISTRO_NAME`). Set to your preferred profile (e.g. `Ubuntu`) so panes inherit your font/colors instead of wt's default. |\n\n## Workflow\n\n1. Write the job spec \u2014 pass it to `dispatch_job`; for a working file use the mission scratch (`write_mission_scratch`), not a loose path.\n2. Call `dispatch_job` (or the CLI). helmd provisions the worktree + branch, compiles the agent config, and launches the harness.\n3. Watch it: `tmux attach -t ship-<slug>`, or in Mission Control. The agent picks up its spec and works autonomously.\n\n## Job Spec Template\n\n```markdown\n# Job Spec: <title>\n\n**Branch:** job/<slug>\n**Agent:** <agent>\n**Mode:** autonomous\n\n## Goal\n\n<one-paragraph outcome>\n\n## File scope\n\n<directories/files the agent may modify>\n\n## What to change\n\n<specific instructions>\n\n## Constraints\n- Key constraints the agent must honor: <hard rules: never weaken guards/tests, never push, gate commands \u2014 or \"none\">\n- (Operator rationale and architecture decisions are not agent payload \u2014 constraints travel; rationale doesn't.)\n\n## Acceptance criteria\n\n<numbered verifiable outcomes>\n\n---\n> Execute this job yourself. Do not delegate the implementation to a subagent \u2014 subagents are only for read-only parallel research, never for writing the code this job asks for.\n>\n> If you notice a bug or problem outside your file scope, jot it as a one-line note in the commit body (`note(out-of-scope): ...`) and continue your work \u2014 don't widen scope to fix it.\n>\n> **When all acceptance criteria are met:** commit cleanly. **DO NOT push the branch.** Pushing to remote is forbidden without the human's explicit say-so. The gate-side process reads the local worktree to discover your work \u2014 remote is irrelevant.\n>\n> (If you're invoked as a pipeline step \u2014 `SHIP_ACTOR_ID` is set, you're inside `ship run` \u2014 then ALSO call `pipelines.step_done` over the daemon RPC surface to advance the step monitor:\n> ```\n> ship ops call pipelines.step_done --json '{\"slug\":\"'$SHIP_ACTOR_ID'\",\"summary\":\"<one-liner>\"}'\n> ```\n> The CLI is a thin transport; everything flows through `ship ops call`. There is no special completion CLI.)\n```\n\n## Test/Impl Split\n\nFor feature work, dispatch as two sequential jobs:\n\n```bash\n# Job 1: tests only (specs live in the mission's scratch \u2014 never a loose .artifacts/)\nbash skills/spawn-agent/scripts/dispatch-portable.sh --slug auth-tests --agent test-writer \\\n  --spec ../helm/data/missions/<mission>/scratch/job-spec-auth-tests.md\n\n# Job 2: implementation (after tests complete and gate passes)\nbash skills/spawn-agent/scripts/dispatch-portable.sh --slug auth-impl --agent rust-lane \\\n  --spec ../helm/data/missions/<mission>/scratch/job-spec-auth-impl.md\n```\n\nThe test spec scopes to test files only. The impl spec references the test files as its contract and must not modify them.\n\n{% if runtime.agents %}\n## Available Agents\n\n{% for a in runtime.agents %}- **{{ a.id }}**{% if a.description %} \u2014 {{ a.description }}{% endif %}\n{% endfor %}\n{% endif %}\n\n## Environment\n\nAll pre-launch verification (CLI on PATH, `ship use <agent>`, `.mcp.json`\nregistration) happens server-side in helmd. A dispatch that fails any\ncheck returns an error from the `Dispatch` RPC; no agent is launched.\nWatch the response: it tells you which check failed.\n\nOverride the worktree base via `SHIP_WORKTREE_DIR` (default: `~/dev/Ship/worktrees`).\nOverride the helmd endpoint via `HELMD_URL=http://host:port` or the shorthand\n`HELMD_ADDR=host:port`. `dispatch-portable.sh` falls back to a local helmd\n(boots `target/debug/helmd` from the repo root or `helmd` on PATH) when\n`HELMD_URL` / `HELMD_ADDR` is unset and the default port is unreachable.\n\n{% if terminal == \"auto\" %}\nTerminal: auto-detected (`$TMUX` \u2192 tmux, `$WT_SESSION` \u2192 wt, `$TERM_PROGRAM` \u2192 iterm/vscode).\n{% else %}\nTerminal: **{{ terminal }}**\n{% endif %}\n\nWorktree base: **{{ worktree_dir }}**\n\n```bash\nship vars set dispatch worktree_dir <path>\n```\n\n## Cleanup\n\nAfter gate passes:\n```bash\ngit worktree remove {{ worktree_dir }}/<slug>\ngit branch -d job/<slug>\n```\n",
});

defineSkill({
  frontmatter: {
    "name": "sui-eval",
    "stable-id": "sui-eval",
    "audience": "internal",
    "description": "Run the SUI authoring eval. Dispatch lane agents to build a view from a spec via dispatch-portable.sh \u2192 helmd, watch their tmux panes for idle (not commits), grade compile + lint + structure, record scores into the sui-eval-runs collection. Off-pipeline \u2014 uses helmd's bare single-agent dispatch, not pipelines.",
    "tags": ["eval", "sui", "dispatch", "grading"],
    "authors": ["ship"],
  },
  assets: [
    { kind: "reference", path: "skills/sui-eval/behavior-spec.md" },
    { kind: "reference", path: "skills/sui-eval/create-space.behavior.jsonc" },
    { kind: "asset", path: "skills/sui-eval/mockups/0-welcome.png" },
    { kind: "asset", path: "skills/sui-eval/mockups/1-name-icon.png" },
    { kind: "asset", path: "skills/sui-eval/mockups/2-theme-chrome.png" },
    { kind: "asset", path: "skills/sui-eval/mockups/3-registered.png" },
    { kind: "script", path: "skills/sui-eval/scripts/behavior-grade.ts" },
    { kind: "script", path: "skills/sui-eval/scripts/dispatch-eval.sh" },
    { kind: "script", path: "skills/sui-eval/scripts/grade.ts" },
    { kind: "script", path: "skills/sui-eval/scripts/lane-watch.sh" },
    { kind: "script", path: "skills/sui-eval/scripts/record.ts" },
    { kind: "script", path: "skills/sui-eval/scripts/template-purity.ts" },
    { kind: "reference", path: "skills/sui-eval/task-spec.md" },
    { kind: "eval", path: "skills/sui-eval/tasks/authority/authority.behavior.jsonc" },
    { kind: "reference", path: "skills/sui-eval/tasks/authority/reference.md" },
    { kind: "reference", path: "skills/sui-eval/tasks/authority/task-spec.md" },
    { kind: "eval", path: "skills/sui-eval/tasks/paired/paired.behavior.jsonc" },
    { kind: "reference", path: "skills/sui-eval/tasks/paired/reference.md" },
    { kind: "reference", path: "skills/sui-eval/tasks/paired/task-spec.md" },
  ],
  body: "# SUI authoring eval\n\nMeasures how well an agent config authors a SUI view from a spec. One lane per\nmodel. No pipeline orchestrator \u2014 `dispatch-portable.sh` POSTs each lane to\nhelmd (boots helmd first if it's down), helmd spawns it in its own worktree +\ntmux tab, a poll loop waits for lanes to idle, then `grade.ts` scores each\nworktree and `record.ts` makes the scores queryable in `views/eval`.\n\n**One dispatch path, no fallback.** Always use `dispatch-portable.sh` \u2014 the\n`dispatch.sh` legacy spawner was deleted. The CLI is a thin\nclient of helmd's `ship.ops.v1.AgentsService/Dispatch`; every consumer\n(this skill, the SPA, the MCP `dispatch_job` tool) routes through it.\n\nWhy off-pipeline: the pipeline fanout path is unreliable (the supervisor sweep\ncan murder live lanes). helmd's bare single-agent dispatch survives daemon\nhiccups and is fully observable in a real PTY.\n\n## Files\n\n- `scripts/dispatch-eval.sh` \u2014 fork N lanes (one per model) from the current branch\n- `scripts/lane-watch.sh` \u2014 peer into each lane's tmux pane, detect idle, auto-grade\n- `scripts/grade.ts` \u2014 compile + lint + structural rubric; writes scores to\n  the run's home (mission scratch when in scope, else `data/runs/sui-eval/<run-id>.jsonl`)\n- `scripts/record.ts` \u2014 jsonl \u2192 `collections.create_item` into `sui-eval-runs`\n- `task-spec.md` + `mockups/` \u2014 the authoring task the lanes build\n\n## Where run output lives\n\nEval runs are **derived, gitignored** \u2014 the durable record is the\n`sui-eval-runs` collection row, not a per-run artifact file. The local\nJSONL the scripts write is a working copy that `record.ts` consumes.\n\nWhen a mission is in scope, the JSONL lands in the mission scratch\n(`${SHIP_MISSION_SCRATCH}/sui-eval-runs/<run-id>.jsonl`). Otherwise the\ncanonical derived-run home is `data/runs/sui-eval/<run-id>.jsonl` (the\nship-meta spec calls `data/runs/` out as the observed-run root). The\nhistorical `.artifacts/sui-eval-runs/` location is retired \u2014 nothing\nwrites there any more.\n\n## Run it\n\n**1. Dispatch.** Haiku-only is the default (cheap smoke). Add models for a live run:\n\n```bash\nbash skills/sui-eval/scripts/dispatch-eval.sh                       # haiku\nbash skills/sui-eval/scripts/dispatch-eval.sh --models haiku,sonnet # live A/B\n```\n\nNote the printed `RUN_ID`. Lanes open as wt tabs, each in a tmux session named\nafter its slug (`eval-<model>-<RUN_ID>`), and work autonomously off `task-spec.md`.\n\n**2. Watch + grade \u2014 one command, no commit race.** Completion is detected by\npeering into the tmux pane, NOT by waiting for a commit. Agents routinely finish\nthe work and never commit; a commit-based watcher reads that as \"no work\" and the\ncohort gets cleaned up. The pane is ground truth \u2014 when Claude Code stops printing\nand its \"esc to interrupt\" indicator is gone, the lane is done:\n\n```bash\nbash skills/sui-eval/scripts/lane-watch.sh <RUN_ID>\n```\n\nIt blocks until every lane is idle (120s quiet + no spinner) or the 40m timeout,\nthen grades each worktree and records \u2014 no external `/loop`, no `git` dependency.\nRun it in a tmux pane of your own (or background it) so it survives. Pass\n`--no-grade` to only signal (writes the per-run done file alongside the jsonl).\n\nIf a lane goes idle with **uncommitted work** on disk \u2014 the case that burns you\ndownstream (idle agent, real work, no commit \u2192 cleanup assumes nothing happened) \u2014\nthe watcher `send-keys` a nudge to commit and gives it another round (up to 2\nprods) before grading the dirty tree anyway. Grading reads files from disk, so an\nuncommitted lane still scores; the prod is about leaving a clean artifact, not\nabout the score.\n\nA lane that's idle but doesn't compile still gets graded \u2014 grade is a measurement,\nnot a gate. Low score is signal, not error.\n\n**3. Read results** in `views/eval` (grouped by run_id) or the raw jsonl under\nthe run's home.\n\n### Manual / headless grade (override the task)\n\n`grade.ts` discovers lanes via `git worktree list` by `-haiku`/`-sonnet`/`-opus`\nsuffix; with zero lane worktrees it falls back to the cwd as one anonymous lane,\nso the whole grading pipeline runs headless against any candidate on disk \u2014 no\ndispatch, no tmux, no daemon. Compile/lint use the prebuilt\n`target/debug/sui-compiler` (override with `SUI_COMPILER=`), never `cargo run`.\n\n```bash\nSHIP_PIPELINE_RUN_ID=<run-id> \\\nSUI_EVAL_VIEW_PATH=views/authority \\\nSUI_EVAL_REQUIRED_FRAGMENTS='{#if,{#for,key={,on:click,{#while-loading,{#while-error' \\\nSUI_EVAL_BEHAVIOR_SPEC=skills/sui-eval/tasks/authority/authority.behavior.jsonc \\\nbun skills/sui-eval/scripts/grade.ts\n```\n\n`SUI_EVAL_BEHAVIOR_SPEC` adds the behavioral dimension: `behavior-grade.ts` runs\nthe spec's scenarios against the candidate in jsdom (vitest + the production WASM\ncompiler via `@ship/test-setup`'s `runScenarios`) and the composite reweights to\ncompile 0.30 / lint 0.10 / structural 0.20 / behavior 0.40. Without it the legacy\n0.40/0.20/0.40 split applies. `behavior-grade.ts` also runs standalone:\n\n```bash\nbun skills/sui-eval/scripts/behavior-grade.ts <lane-dir> <view-path> <spec.behavior.jsonc>\n```\n\n## Cleanup\n\nhelmd's dispatch lifecycle owns lane teardown: worktrees, branches, and\nthe post-run `.artifacts/` were never yours to clean up. If a lane is\nstill alive at the end of a run, `tmux kill-session -t <slug>` is the\nonly operator-side cleanup; everything else is helmd's.\n",
});

defineSkill({
  frontmatter: {
    "name": "tdd",
    "audience": "public",
    "stable-id": "tdd",
    "description": "Use when implementing any feature, fix, or behavior change. Enforces the TDD protocol \u2014 write the failing test first, then the minimal code to pass it. Includes interactive spec-building workflow for collaborative test design.",
    "tags": ["testing", "workflow", "engineering", "tdd", "spec"],
    "authors": ["ship"],
    "artifacts": ["markdown"],
  },
  assets: [
    { kind: "asset", path: "skills/tdd/assets/vars.json" },
    { kind: "eval", path: "skills/tdd/evals/evals.json" },
    { kind: "reference", path: "skills/tdd/references/docs/green.md" },
    { kind: "reference", path: "skills/tdd/references/docs/index.md" },
    { kind: "reference", path: "skills/tdd/references/docs/interactive-workflow.md" },
    { kind: "reference", path: "skills/tdd/references/docs/red.md" },
    { kind: "reference", path: "skills/tdd/references/docs/refactor.md" },
  ],
  body: "{% if phase == \"red\" %}\n# Red Phase\n\nYou are writing a failing test. This test defines a behavior that does not exist yet.\n\n**Rules:**\n- Write ONE test at a time\n- Run it \u2014 it MUST fail\n- If it passes, the test is wrong or the behavior already exists. Delete it and try again.\n- Edit test files only. Do NOT touch implementation files.\n\n{% if test_style == \"integration\" %}\nWrite an **integration test** \u2014 crosses real boundaries (DB, filesystem, network). No mocks for the system under test. Fail because the behavior doesn't exist, not because of missing infrastructure.\n{% elif test_style == \"e2e\" %}\nWrite an **end-to-end test** \u2014 full user flow through real endpoints. HTTP, browser, or CLI. No internal shortcuts.\n{% else %}\nWrite a **unit test** \u2014 isolated, no I/O, fast. Mock external dependencies. One function per test.\n{% endif %}\n\nCheck `${EVIDENCE_DIR:-${SHIP_MISSION_SCRATCH:-.}/}spec-from-tests.md` for the\nplanned test list if it exists. The evidence home is the **mission scratch**\nwhen a dispatch is in scope (the spec travels with the mission), else the\njob's working dir root. The historical `.artifacts/spec-from-tests.md`\nlocation is retired \u2014 do not write there.\n{% elif phase == \"green\" %}\n# Green Phase\n\nThere is a failing test. Your only job is to make it pass.\n\n**Rules:**\n- Read the failing test first \u2014 understand exactly what it expects\n- Write the MINIMUM code to make it pass\n- Do NOT modify test files\n- Do not add features, error handling, or edge cases the test doesn't require\n- Do not refactor\n\n{% if test_style == \"integration\" %}\nMinimum still means real I/O \u2014 you can't stub the database to pass an integration test. But hardcode return values, skip validation, skip edge cases. The next Red test forces each of those.\n{% elif test_style == \"e2e\" %}\nWire only the happy path the test exercises. Leave error handling, middleware, and edge cases for later Red tests.\n{% else %}\nIf the test expects `return 42`, write `return 42`. The next Red test forces generalization. That's the point.\n{% endif %}\n\n**Process:** Read test \u2192 write minimum code \u2192 run it \u2192 if green{% if commit_at_green %}, commit{% endif %} and hand off to Refactor \u2192 if not, adjust and repeat.\n\n{% elif phase == \"refactor\" %}\n# Refactor Phase\n\nAll tests are green. Make the code better without changing what it does.\n\n**Rules:**\n- Run ALL tests before starting \u2014 they must pass\n- Run ALL tests after every change \u2014 they must still pass\n- If a test fails after your change, revert immediately\n- Do NOT add behavior, new tests, or new dependencies\n\n**What to improve:** duplication, naming, complex conditionals, oversized functions, dead code, formatting, nesting depth.\n\n{% if test_style == \"e2e\" %}\nE2e tests only see the public surface \u2014 restructure internals freely as long as the endpoint contract holds.\n{% elif test_style == \"integration\" %}\nIntegration tests pin the public API. Renaming internals is safe. Changing the shape of what the test calls is not.\n{% endif %}\n\n**Process:** Confirm green \u2192 pick one improvement \u2192 change \u2192 confirm still green \u2192 commit \u2192 repeat or stop.\n\n{% else %}\n# TDD\n\nWrite the test first. Watch it fail. Write the minimum code to pass. Refactor.\n\nIf you didn't watch it fail, you don't know if it tests the right thing.\n\n## The Loop\n\n```\n1. Write a failing {{ test_style }} test  \u2192  RED\n2. Confirm it fails for the right reason\n3. Write minimum code to pass            \u2192  GREEN\n4. Confirm it passes\n5. Refactor if needed                    \u2192  REFACTOR\n{% if commit_at_green %}6. Commit\n{% endif %}7. Repeat\n```\n\n**No production code before a failing test.** No exceptions.\n\n{% if test_style == \"integration\" %}\n## Test Style: integration\n\nIntegration tests \u2014 cross real boundaries (DB, filesystem, network). No mocks for the system under test. Slower, but they catch wiring bugs unit tests miss.\n{% elif test_style == \"e2e\" %}\n## Test Style: e2e\n\nEnd-to-end tests \u2014 full user flows through real endpoints. Nothing is stubbed. Highest confidence, slowest feedback loop.\n{% else %}\n## Test Style: unit\n\nUnit tests \u2014 isolated, no I/O, fast. Mock external dependencies. One function or module per test.\n{% endif %}\n\n## Phase Rules\n\n**Red:** ONE test at a time. Edit test files only. Confirm the right failure before moving on.\n\n**Green:** Read the failing test first. Write the minimum \u2014 if the test expects `42`, return `42`. The next test forces generalization. Do not refactor.{% if commit_at_green %} Commit at green.{% endif %}\n\n**Refactor:** Run all tests before and after every change. Revert immediately if anything breaks. No new behavior.\n\n## Spec Artifact\n\nAfter 5\u20138 tests, write the spec artifact to the **mission scratch** (or the\njob's working dir root if no mission is in scope). The historical\n`.artifacts/` write target is retired \u2014 do not write there.\n\n```bash\n# Evidence home: mission scratch first (so the spec travels with the\n# mission), else the working dir root. Hard-fail-soft when a dispatch is\n# in scope and SHIP_MISSION_SCRATCH is unset \u2014 fall back to writing into\n# the wrong place and the next dispatch's report never lines up.\nif [[ -n \"${SHIP_MISSION_SCRATCH:-}\" ]]; then\n    EVIDENCE_DIR=\"$SHIP_MISSION_SCRATCH\"\nelse\n    EVIDENCE_DIR=\"$(pwd)\"\nfi\nmkdir -p \"$EVIDENCE_DIR\"\ncat > \"$EVIDENCE_DIR/spec-from-tests.md\" <<'EOF'\n# Spec (from tests)\n\nBehaviors confirmed:\n- [x] <behavior> \u2014 <test name>\n\nOpen questions:\n- <decision not yet pinned>\n\nNext tests:\n- <next behavior to write>\nEOF\n```\n\nThis is the deliverable of the spec phase. Implementation agents use it as their brief.\n## Interactive Spec-Building\n\nFor collaborative test design \u2014 proposing tests one at a time, discussing implications, checkpointing specs \u2014 see `references/docs/interactive-workflow.md`.\n\n{% endif %}\n\n## Running Tests\n\n{% if test_runner == \"cargo\" %}\n`cargo test` \u2014 run full suite. `cargo test <name>` \u2014 run one test. `cargo test -- --nocapture` \u2014 with output.\n{% elif test_runner == \"vitest\" %}\n`npx vitest run` \u2014 full suite. `npx vitest run <file>` \u2014 one file. `npx vitest run --reporter=verbose` \u2014 verbose.\n{% elif test_runner == \"jest\" %}\n`npx jest` \u2014 full suite. `npx jest <file>` \u2014 one file. `npx jest --verbose` \u2014 verbose.\n{% elif test_runner == \"pytest\" %}\n`pytest` \u2014 full suite. `pytest <file>::<name>` \u2014 one test. `pytest -v` \u2014 verbose.\n{% elif test_runner == \"go\" %}\n`go test ./...` \u2014 full suite. `go test -run <name> ./...` \u2014 one test. `go test -v ./...` \u2014 verbose.\n{% elif test_runner == \"make\" %}\n`make test` \u2014 full suite. `make test VERBOSE=1` \u2014 verbose.\n{% else %}\nDetect from project files:\n\n| File | Run one test | Full suite |\n|------|-------------|------------|\n| `Cargo.toml` | `cargo test <name>` | `cargo test` |\n| `package.json` + vitest | `npx vitest run <file>` | `npx vitest run` |\n| `package.json` + jest | `npx jest <file>` | `npx jest` |\n| `pyproject.toml` | `pytest <file>::<name>` | `pytest` |\n| `go.mod` | `go test -run <name> ./...` | `go test ./...` |\n| `Makefile` | `make test` | `make test` |\n{% endif %}\n",
});

defineSkill({
  frontmatter: {
    "name": "trigger-authoring",
    "audience": "internal",
    "stable-id": "trigger-authoring",
    "description": "Author Ship triggers \u2014 JSONC files in .ship/triggers/*.jsonc that match a Ship event and run actions. Reactive automation layer. Distinct from agent/provider hooks, which are a compiler concern.",
    "tags": ["triggers", "extension-authoring", "jsonc", "jinja", "automation"],
    "authors": ["ship"],
  },
  body: "# Ship Triggers \u2014 The Reactive Automation Layer\n\nA **trigger** is a declarative JSONC file that listens for a Ship event and runs one or more actions when it fires. You write it as `.ship/triggers/<name>.jsonc`, the daemon hot-reloads on file change, and your automation is live.\n\n> **Naming note.** Internal Rust types still carry the `Hook*` prefix (`HookDef`, `HookEngine`, `HookAction`) for historical reasons; the public/filesystem surface is consistently **triggers**. \"Hooks\" elsewhere in Ship refers to the agent/provider compiler hooks (pre/post-tool), a different system entirely. Keep the two separate in your head.\n\n## Minimal example\n\n`.ship/triggers/log-failed-jobs.jsonc`:\n\n```jsonc\n{\n  \"name\": \"log-failed-jobs\",\n  \"description\": \"Emit a follow-up event when a job ends in failure.\",\n  \"trigger\": {\n    \"event\": \"job.failed\"\n  },\n  \"actions\": [\n    {\n      \"type\": \"event.emit\",\n      \"event_type\": \"trigger.job_failed_notice\",\n      \"payload\": {\n        \"job_id\": \"{{ event.entity_id }}\",\n        \"triggered_by\": \"{{ event.id }}\"\n      }\n    }\n  ]\n}\n```\n\nDrop the file in `.ship/triggers/`, the daemon watcher picks it up, and the next `job.failed` event produces a `trigger.job_failed_notice` event with the rendered payload.\n\n## Definition shape\n\n```jsonc\n{\n  \"name\": \"<unique slug>\",          // required \u2014 used in logs and dedup\n  \"description\": \"<sentence>\",      // optional but strongly encouraged\n  \"trigger\": {\n    \"event\": \"<event.type>\",        // required \u2014 see \u00a7Events below\n    \"filter\": { ... }               // optional \u2014 see \u00a7Filters\n  },\n  \"actions\": [                      // 1+ actions, run sequentially\n    { \"type\": \"<action>\", ... }\n  ]\n}\n```\n\n## Events\n\nEvery Ship event is a trigger candidate. Pick from these namespaces:\n\n| Namespace | Events | Typical use |\n|---|---|---|\n| `workspace.*` | `created`, `deleted`, `compiled`, `started`, `branch_switched` | Snapshot state, warm caches |\n| `session.*` | `started`, `completed`, `failed`, `cancelled`, `suspended` (+ `session.turn.*`) | Digest daily activity |\n| `actor.*` | `created`, `woke`, `slept`, `stopped`, `crashed` | Alert on agent crashes |\n| `job.*` | `created`, `claimed`, `completed`, `failed` | Triage on failure, archive completed |\n| `pipeline.*` | `started`, `step_completed`, `step_failed`, `blocked`, `completed` | Pipeline notifications |\n| `collection.*` | `item_created`, `item_updated`, `item_deleted`, `query_executed` | Derived data, search indexing |\n| `skill.*` | `started`, `completed`, `failed` | Skill-usage metrics |\n| `gate.*` | `passed`, `failed` | Auto-merge on pass, flag on fail |\n| `webhook.received` | inbound from external providers (Telegram, GitHub, etc.) | Integration glue |\n| `connection.*` | `created`, `expired` | Re-auth prompts |\n| `config.changed` | `.ship/` files changed | Warm reloads |\n\nCanonical event-type constants live in `crates/core/ship-types/src/event_types.rs`.\n\n## Filters\n\nA filter narrows which event instances fire the trigger. Dot-path keys match against the event payload:\n\n```jsonc\n\"trigger\": {\n  \"event\": \"job.failed\",\n  \"filter\": {\n    \"payload.pipeline_name\": \"ship-dev\",\n    \"payload.retry_count\": { \"$gte\": 3 }\n  }\n}\n```\n\nFilter operators:\n- Equality: `\"key\": \"value\"` or `{ \"$eq\": \"value\" }`\n- Comparison: `{ \"$gt\": N }`, `{ \"$gte\": N }`, `{ \"$lt\": N }`, `{ \"$lte\": N }`, `{ \"$ne\": X }`\n- Set: `{ \"$in\": [a, b, c] }`, `{ \"$nin\": [...] }`\n- Existence: `{ \"$exists\": true }`\n- Array contains: `{ \"$contains\": \"x\" }`\n- AND is implicit across keys. For OR, define two triggers.\n\n## Actions\n\nFour action types are implemented today. More arrive with Phase 2 (connections + notifications).\n\n### `collection.insert`\n```jsonc\n{\n  \"type\": \"collection.insert\",\n  \"collection\": \"bug_triage\",\n  \"data\": {\n    \"title\": \"{{ event.payload.issue.title }}\",\n    \"severity\": \"{{ 'high' if 'critical' in event.payload.issue.labels else 'normal' }}\",\n    \"source_event\": \"{{ event.id }}\"\n  }\n}\n```\nOr, for multiline JSON, reference a `.j2` file:\n```jsonc\n{\n  \"type\": \"collection.insert\",\n  \"collection\": \"bug_triage\",\n  \"template_file\": \"triggers/templates/triage-bug.j2\"\n}\n```\nThe template must render to a valid JSON object.\n\n### `collection.update`\n```jsonc\n{\n  \"type\": \"collection.update\",\n  \"collection\": \"tasks\",\n  \"filter\": { \"id\": \"{{ event.entity_id }}\" },\n  \"data\": { \"status\": \"done\", \"completed_at\": \"{{ now() }}\" }\n}\n```\n\n### `event.emit`\n```jsonc\n{\n  \"type\": \"event.emit\",\n  \"event_type\": \"trigger.digest_ready\",\n  \"payload\": { \"summary\": \"...\" }\n}\n```\nEmitted events can themselves trigger other triggers \u2014 see \u00a7Recursion guard.\n\n### `pipeline.trigger`\n```jsonc\n{\n  \"type\": \"pipeline.trigger\",\n  \"pipeline\": \"bug-triage\",\n  \"args\": { \"issue_url\": \"{{ event.payload.issue.html_url }}\" }\n}\n```\nUse this as the escape hatch for anything that needs multi-step async work or agent involvement. If Jinja + one-step actions aren't enough, trigger a pipeline and let the pipeline handle complexity.\n\n### Conditional actions\n\nEvery action accepts an optional `when` Jinja expression. Falsy \u2192 skip:\n\n```jsonc\n{\n  \"type\": \"pipeline.trigger\",\n  \"pipeline\": \"escalate\",\n  \"when\": \"{{ 'critical' in event.payload.labels }}\",\n  \"args\": { ... }\n}\n```\n\n## Jinja rendering context\n\nInside `data`, `payload`, `filter`, `template_file` content, and `when`:\n\n| Variable | Contents |\n|---|---|\n| `event` | The full event envelope: `id`, `type`, `entity_id`, `payload`, `occurred_at` |\n| `workspace` | Current workspace metadata |\n| `actor` | Actor that caused the event, when available |\n| `results` | Array of prior-action results in this trigger, indexed by position. `{{ results[0].id }}` after a `collection.insert` |\n| `now()` | Current UTC timestamp (ISO 8601) |\n\nThe renderer is a fresh `minijinja::Environment` per evaluation \u2014\ncustom filters are not registered today, so `{{ a | b }}` chains\nlimited to minijinja built-ins (`length`, `upper`, `lower`, `default`,\netc.). The renderer lives at\n`crates/core/ship-platform/src/triggers/render.rs` \u2014 register a filter\nthere if you genuinely need one.\n\n## Recursion guard\n\nTriggers can emit events that fire other triggers. To prevent infinite loops the engine enforces **max depth 3**. Beyond depth 3, further trigger invocations are dropped and a `trigger.depth_exceeded` diagnostic event is emitted.\n\n## What triggers cannot do\n\n- **No shell execution.** No `sh`, no `eval`, no arbitrary commands. If you need a shell command, trigger a pipeline that runs it.\n- **No raw HTTP.** The `connection.proxy` and `notify` actions (Phase 2) are the only way to hit external services. Until they land, triggers stay local.\n- **No custom event types.** Only the canonical event namespaces match. You can *emit* a `trigger.*` event, but agents can't invent new top-level event namespaces.\n- **No ordering guarantees across triggers.** Two triggers on the same event run concurrently. If ordering matters, compose one trigger or use a pipeline.\n\n## Validation\n\nThe daemon validates every trigger on load and on file change:\n- `trigger.event` must be a known event type\n- Each `action.type` must be a known action\n- Referenced `template_file` paths must exist and parse\n- Jinja expressions must compile (actual evaluation is lazy)\n\nInvalid triggers log a warning and are skipped \u2014 they do not crash the daemon.\n\n## Authoring workflow\n\n1. Write the JSONC file in `.ship/triggers/<name>.jsonc`\n2. Confirm the daemon loaded it: `tail -n 20 ~/.ship/logs/shipd.stderr.log | grep trigger`\n3. Fire the event manually for testing \u2014 via the CLI (`ship events emit <type>`) or by producing it naturally\n4. Inspect the result (collection write, emitted event, pipeline run) via the events view or Studio\n\n## Common patterns\n\n**On job failure, notify and archive:**\n```jsonc\n{\n  \"name\": \"triage-failed-jobs\",\n  \"trigger\": { \"event\": \"job.failed\" },\n  \"actions\": [\n    { \"type\": \"collection.insert\", \"collection\": \"failed_jobs\",\n      \"data\": { \"job_id\": \"{{ event.entity_id }}\", \"reason\": \"{{ event.payload.reason }}\" } },\n    { \"type\": \"event.emit\", \"event_type\": \"trigger.job_failure_archived\",\n      \"payload\": { \"job_id\": \"{{ event.entity_id }}\" } }\n  ]\n}\n```\n\n**On external webhook, route to a pipeline:**\n```jsonc\n{\n  \"name\": \"github-issue-triage\",\n  \"trigger\": {\n    \"event\": \"webhook.received\",\n    \"filter\": { \"payload.provider\": \"github\", \"payload.action\": \"opened\" }\n  },\n  \"actions\": [\n    { \"type\": \"pipeline.trigger\", \"pipeline\": \"triage\",\n      \"args\": { \"issue_url\": \"{{ event.payload.issue.html_url }}\" } }\n  ]\n}\n```\n\n**On collection write, derive a second collection item:**\n```jsonc\n{\n  \"name\": \"index-notes\",\n  \"trigger\": {\n    \"event\": \"collection.item_created\",\n    \"filter\": { \"payload.collection\": \"notes\" }\n  },\n  \"actions\": [\n    { \"type\": \"collection.insert\", \"collection\": \"note_search_index\",\n      \"data\": {\n        \"note_id\": \"{{ event.payload.item_id }}\",\n        \"title\": \"{{ event.payload.data.title }}\",\n        \"excerpt\": \"{{ event.payload.data.content[:200] }}\"\n      } }\n  ]\n}\n```\n\n## Relation to other systems\n\n- **Views** can read a collection that a trigger writes to, giving you live dashboards of derived state. Views subscribe via `<data name=\"\u2026\" source=\"ship.collection.items(\u2026)\" live />` and stay current as triggers mutate the table. See `ship-views`.\n- **Pipelines** handle multi-step or agent-involving work. Triggers are the cheap reflex; pipelines are the thoughtful response.\n- **Connections** (Phase 2) will add `notify` and `connection.proxy` actions so triggers can send messages and call external APIs through authenticated proxies.\n\n## Do not confuse with\n\n- `**/hooks.toml`, `.claude/hooks/`, provider-level pre/post-tool hooks \u2014 those are **agent/provider hooks**, a compiler concern for Claude/Codex/Cursor/Gemini/OpenCode. Completely separate system that happens to share a name.\n",
});

defineSkill({
  frontmatter: {
    "name": "web-qa",
    "audience": "public",
    "stable-id": "web-qa",
    "description": "Systematic QA for a UI surface \u2014 every interactive element pressed, multi-theme, console + network captured, evidence written to `${SHIP_MISSION_SCRATCH}/web-qa/` (or `.artifacts/web-qa/` when no mission scratch is set). Use before claiming any UI work \"done\". Gate-required when touching apps/shell, packages/components/{sui,ship}, views, or `sui/crates/sui`.",
    "tags": ["qa", "testing", "browser", "playwright", "gate-required"],
    "authors": ["ship"],
    "license": "MIT",
    "artifacts": ["image", "markdown", "jsonl"],
    "allowed-tools": ["Bash(playwright-cli*)", "Bash(npx playwright*)", "Bash(bunx playwright*)", "Bash(curl*)", "Read", "Write"],
  },
  assets: [
    { kind: "asset", path: "skills/web-qa/assets/vars.json" },
  ],
  body: "# Web QA \u2014 gated UI evidence\n\nYou are not done with UI work until this skill has produced a fresh artifact for\nthe surface you touched. Taking a screenshot and saying \"looks fine\" is not QA.\nReal QA exercises every declared interactive element, in every declared theme,\ncaptures the console and network for the whole run, and verifies that elements\nboth exist in the DOM **and** render with non-zero box, non-hidden visibility,\nand >0 opacity.\n\nIf you skip this skill, the gate will reject your branch.\n\n## Setup\n\n```bash\nREPO=\"$(git rev-parse --show-toplevel)\"\nQA_RUN=\"$(date -u +%Y-%m-%dT%H-%M)\"\nQA_SURFACE=\"${1:?surface name required, e.g. dev-panel/edit-mode/components-tab}\"\n# Evidence homes \u2014 mission scratch when one is in scope, repo-local .artifacts/\n# otherwise. The gate reads from this exact path; rotating the dir breaks the\n# gate scan. Hard-fail when the dispatch carried a mission but\n# SHIP_MISSION_SCRATCH is unset \u2014 fall back to writing into the wrong place\n# silently and the next dispatch's report never lines up.\nif [[ -n \"${SHIP_MISSION_SCRATCH:-}\" ]]; then\n    QA_DIR=\"$SHIP_MISSION_SCRATCH/web-qa/$QA_SURFACE/$QA_RUN\"\nelse\n    QA_DIR=\"$REPO/.artifacts/web-qa/$QA_SURFACE/$QA_RUN\"\nfi\nmkdir -p \"$QA_DIR/screenshots\"\n\n# Resolve playwright-cli \u2014 see references/runtime.md for fallback walk\nif command -v playwright-cli &>/dev/null; then PW=\"playwright-cli\"\nelif command -v bunx &>/dev/null; then PW=\"bunx playwright-cli\"\nelse PW=\"npx playwright-cli\"; fi\n$PW --version >/dev/null || { echo \"playwright-cli unavailable\" >&2; exit 1; }\n```\n\nUse `$PW` for every browser command and `$QA_DIR` for every path. Never use\nrelative paths \u2014 playwright-cli resolves from its own working directory.\n\n## Required upfront declaration\n\nBefore touching the browser, write `$QA_DIR/plan.md` declaring:\n\n1. **Surface name** \u2014 `dev-panel/edit-mode/components-tab`, `views/agents/list`, etc. Use slashes for hierarchy.\n2. **Inventory** \u2014 every interactive element on the surface (button, input, tab, link, drag handle, keyboard shortcut). One per row. Include declared aria-labels.\n3. **States** \u2014 every state the surface can be in (empty, loading, populated, error, hover, focused, disabled, overflow). One per row.\n4. **Theme matrix** \u2014 at minimum `ship-light`, `ship-dark`, `console`. Other registered themes: `paper`, `halo`, `contrast`, `glass` (see `sui/packages/sui/tokens/src/themes.ts`). Add a theme to the matrix when the surface has theme-conditional CSS.\n5. **Viewport matrix** \u2014 at minimum `1280x720` (desktop) and `375x812` (mobile-narrow). Add others when the surface is responsive-specific.\n\nIf you cannot declare these without invention, you don't understand the surface\nwell enough to QA it. Stop and re-read the source.\n\n## Per-element probe protocol\n\nFor every element \u00d7 every state \u00d7 every theme:\n\n```bash\n# 1. Set theme (project-specific \u2014 themes are toggled via localStorage)\n$PW localstorage-set ship-site-theme \"$THEME\"\n$PW reload\n\n# 2. Pre-press evidence\n$PW snapshot --filename=\"$QA_DIR/snap-$ELEMENT-$STATE-$THEME-before.yml\"\n$PW screenshot --filename=\"$QA_DIR/screenshots/$ELEMENT-$STATE-$THEME-before.png\"\n\n# 3. The press itself (one of: click, fill, press, hover, drag)\n$PW click \"$REF\"\n\n# 4. Post-press evidence\n$PW snapshot --filename=\"$QA_DIR/snap-$ELEMENT-$STATE-$THEME-after.yml\"\n$PW screenshot --filename=\"$QA_DIR/screenshots/$ELEMENT-$STATE-$THEME-after.png\"\n\n# 5. Visibility assertion \u2014 the failure mode I keep shipping past\n$PW --raw eval \"(() => {\n  const el = $LOCATOR;\n  if (!el) return JSON.stringify({ok:false, reason:'not in DOM'});\n  const r = el.getBoundingClientRect();\n  const cs = getComputedStyle(el);\n  return JSON.stringify({\n    ok: r.width > 0 && r.height > 0 && cs.visibility !== 'hidden' && cs.display !== 'none' && parseFloat(cs.opacity) > 0,\n    rect: {w:r.width, h:r.height},\n    visibility: cs.visibility, display: cs.display, opacity: cs.opacity,\n  });\n})()\"\n\n# 6. Console + network capture after each interaction\n$PW console --errors >> \"$QA_DIR/console.log\"\n$PW requests >> \"$QA_DIR/network.log\"\n\n# 7. Append one line to evidence.jsonl per probe\njq -nc \\\n  --arg element \"$ELEMENT\" --arg state \"$STATE\" --arg theme \"$THEME\" \\\n  --arg pre \"$QA_DIR/screenshots/$ELEMENT-$STATE-$THEME-before.png\" \\\n  --arg post \"$QA_DIR/screenshots/$ELEMENT-$STATE-$THEME-after.png\" \\\n  --argjson visibility \"$VISIBILITY_JSON\" \\\n  '{element:$element, state:$state, theme:$theme, pre:$pre, post:$post, visibility:$visibility, ts: now|todate}' \\\n  >> \"$QA_DIR/evidence.jsonl\"\n```\n\n**Element-rendered check (step 5) is the load-bearing one.** A `chipCount: 13`\nin shadow DOM means nothing if every chip has `display: none` from a CSS rule\nthat doesn't reach into the slot. The visibility assertion catches this.\n\n## Console + network gates\n\n```bash\n# Any console error or warning fails the run unless explicitly waived in report.md\n$PW console >> \"$QA_DIR/console.log\"\n$PW console --errors > \"$QA_DIR/console-errors.log\"\n$PW console --warnings > \"$QA_DIR/console-warnings.log\"\n\n# Any non-2xx network response fails the run unless explicitly waived\n$PW requests > \"$QA_DIR/network.log\"\n```\n\n**A waiver is a one-line entry in `report.md` under `## Waivers` with a reason.**\nExamples of legitimate waivers: third-party tracker pings, dev-mode HMR\nheartbeats. \"Looks fine\" is not a reason.\n\n## Theme matrix\n\nFor each theme in the matrix:\n\n```bash\nfor THEME in ship-light ship-dark console; do\n  $PW localstorage-set ship-site-theme \"$THEME\"\n  $PW reload\n  # Confirm the switch took \u2014 silently-not-applied is a class of failure\n  APPLIED=$($PW --raw eval \"document.documentElement.dataset.theme\")\n  [ \"$APPLIED\" = \"\\\"$THEME\\\"\" ] || echo \"WARN: theme did not apply: set=$THEME got=$APPLIED\" >&2\n  $PW screenshot --filename=\"$QA_DIR/screenshots/_baseline-$THEME.png\"\n  # ... full per-element protocol above ...\ndone\n```\n\nThe baseline screenshot is your eyeball check for theme-aware regressions\n(invisible-against-bg text, contrast loss, color-mix() unwinding wrong).\n\n## Viewport matrix\n\n```bash\nfor VP in \"1280x720\" \"375x812\"; do\n  $PW resize ${VP%x*} ${VP#*x}\n  $PW screenshot --filename=\"$QA_DIR/screenshots/_baseline-$VP.png\"\n  # ... full protocol ...\ndone\n```\n\nCapture overflow + truncation behavior at each width.\n\n## report.md format\n\nEvery run ends with `$QA_DIR/report.md`:\n\n```markdown\n---\nsurface: dev-panel/edit-mode/components-tab\nstatus: DONE | DONE_WITH_CONCERNS | BLOCKED | NEEDS_CONTEXT\nrun: 2026-05-16T14-30\nelements_probed: 12\nthemes_covered: ship-light, ship-dark, ship-console\nviewports_covered: 1280x720, 375x812\nconsole_errors: 0\nconsole_warnings: 1\nnetwork_failures: 0\nhealth_score: 87\n---\n\n## Summary\n\n<1-2 sentences: what surface, what tier, top finding.>\n\n## Findings\n\nFor each failing probe:\n\n### F-001 \u2014 [severity] component does not render with non-zero box in ship-dark\n- **Element:** `<sui-button slot=\"action\">` ref `e15`\n- **State:** populated\n- **Theme:** ship-dark\n- **Viewport:** 1280x720\n- **Visibility check:** `{ok: false, rect: {w:0, h:0}, display: 'flex'}`\n- **Evidence:** screenshots/e15-populated-ship-dark-after.png\n- **Reproduction:**\n  1. `playwright-cli goto http://localhost:3270/dev-panel`\n  2. `playwright-cli localstorage-set ship-site-theme ship-dark`\n  3. `playwright-cli reload`\n  4. `playwright-cli click e3` (open Components tab)\n  5. Observe: tab has slot content but content has zero size\n- **Suspected cause:** host stylesheet defines `.components-tab > *` but the\n  slotted children render inside `<sui-va-components>` shadow root, beyond\n  reach. Move the rule into the inner component's :host stylesheet.\n\n## Waivers\n\n<Empty by default. Every waiver is one line + reason.>\n\n## Health score\n\n| Category      | Weight | Score |\n|---------------|--------|-------|\n| Console clean | 15%    | 100   |\n| Network clean | 10%    | 100   |\n| Rendered      | 20%    | 75    |\n| Themed        | 20%    | 80    |\n| Responsive    | 10%    | 90    |\n| A11y          | 15%    | 70    |\n| Interactive   | 10%    | 100   |\n| **Total**     | 100%   | **87** |\n```\n\nA `health_score` \u2265 90 is shippable. 70\u201389 is `DONE_WITH_CONCERNS`. <70 is\n`BLOCKED`.\n\n## Completion status protocol\n\nEvery run ends with one of:\n\n- **DONE** \u2014 every probe passed, score \u2265 90, console + network clean.\n- **DONE_WITH_CONCERNS** \u2014 passed but with caveats. List each concern as a\n  finding even if you couldn't fix it this turn.\n- **BLOCKED** \u2014 something prevents completion (daemon down, surface 404s,\n  required state unreachable). State the blocker + what you tried + what you\n  recommend.\n- **NEEDS_CONTEXT** \u2014 you don't know enough about the surface to declare its\n  inventory. State exactly what's missing.\n\nLying \u2014 declaring DONE when there are unprobed elements or unaddressed\nfindings \u2014 is a worse failure than honest `DONE_WITH_CONCERNS`.\n\n## Gate contract\nThe gate scans the evidence home \u2014 `${SHIP_MISSION_SCRATCH}/web-qa/<surface>/<latest>/report.md` when a dispatch carried a mission, else `.artifacts/web-qa/<surface>/<latest>/report.md`. The gate checks the\n`run:` timestamp is newer than the latest commit touching the matched files,\nand parses `status:` + `console_errors:` + `network_failures:` from the\nfrontmatter. Missing or stale artifact = gate FAIL.\n  composed-event paths don't fire from synthetic `.click()` in shadow DOM)\n- Marking DONE when console has unwaived errors\n- Marking DONE when the score is < 90 without justifying every dropped point\n\n## References\n\n- `references/playwright-cli.md` \u2014 every playwright-cli verb worth knowing\n- `references/health-rubric.md` \u2014 full scoring math\n- `references/element-locators.md` \u2014 when to use ref vs selector vs role vs\n  testid, and how to find clickable divs the a11y tree misses\n",
});
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
