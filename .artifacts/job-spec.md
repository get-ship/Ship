# Job Spec: ship CLI wide cull — collapse to op-client + operator + dev-tooling

**Branch:** job/cli-wide-cull  (fork from launch/unified-HEAD @ f01cab1fa, post cli-cull + substrate merges)
**Agent:** rust-lane
**Mode:** autonomous

## Premise (operator intent — do not re-litigate)
`ship` is NOT a product CLI. Its tagline "Agent configuration compiler — compose,
compile, distribute" is the pre-pivot product. Post-pivot: Ship = agentic-app runtime;
Floki + UI = product; ConnectRPC is the only surface Ship exposes. A CLI command that
does local disk/db work instead of calling a Connect op is legacy by definition. This
job collapses `ship` to three legitimate roles and deletes everything else.

## SURVIVOR SET — KEEP exactly these (whitelist; everything else goes)
1. **Agent surface:** `ops` (list/call) — the generic Connect client. Untouchable.
2. **Operator:** the `daemon` family (start/stop/restart/status/dir/cleanup/run/
   supervise/install-service/uninstall-service), `doctor`, `meta`. Process/host control
   + diagnostics — legitimately not a Connect call.
3. **Dev-tooling (NOT product — the omp/dispatch workflow depends on these):**
   `use`, `compile`, `config`. `dispatch.sh` calls `ship use <agent>` (compiles the agent
   → CLAUDE.md, the boot of every dispatched agent) and `ship config get`. Keep them; they
   are flagged for eventual migration to daemon-side dispatch, but cutting them now breaks
   the agent loop. Plus local UX: `docs`, `help`, and `build` (the `just build` wrapper).

## CUT — delete these commands + their handlers (the wide cull)
For each: remove the `Commands` variant in `cli.rs`, the dispatch arm + `mod` decl in
`main.rs`, any subcommand enum in `commands.rs`, the handler module(s), and their tests.

- `init`            → init.rs (scaffolds .ship/ on disk; SpaceService territory)
- `status`          → the Status arm (compiler-product state readout; keep profile.rs only
                       insofar as `use` needs it — see SHARED MODULES)
- `validate`        → validate.rs, validate_tests.rs
- `install`         → install.rs
- `add`             → add.rs, add/ (whole dir)
- `publish`         → publish.rs, publish_tests.rs
- `mcp`             → mcp.rs + McpCommands (McpService owns this)
- `isolation`       → isolation_cmd.rs + IsolationCommands (IsolationService owns this)
- `run`            → ship_run.rs, ship_run_tests.rs (PipelinesService.Dispatch / ops)
- `procs`          → procs.rs (RuntimeService.ListProcs / ops)
- `tail`           → tail.rs (EventsService.Poll / ops)
- `sui`            → sui.rs + SuiCommands (UiService; a ship-code/editor concern)
- `events`         → events_cmd.rs + EventsCommands (EventsService / ops)
- `audit`          → audit.rs (real enforcement is at the gated-op layer)
- `workspace`      → trash_cmd.rs + WorkspaceCommands
- `auth` + `login` + `logout` + `whoami` → auth.rs (whole file) + AuthCommands.
                       Operator intent: **auth is not a CLI concern** — it belongs in the
                       UI/Studio. Removes login/logout/whoami/pair/status/revoke wholesale.
- `hook`           → hook.rs + HookCommands. Operator intent: **hooks are a ship-code
                       concern.** Cut from `ship`; the tool-gate capability moves to
                       ship-code (note it in the commit body; do NOT build it here).

(`convert`, `runtime`, `uninstall`, `top`, `rebuild-projections`, `add --from`, and the
auth-session REST were already removed in the prior cli-cull merge — confirm they're gone,
don't re-do.)

## SHARED MODULES — caution
- `profile.rs` and `compile/` are used by BOTH the kept `use`/`compile` AND the cut
  `status`/`init`. KEEP the parts `use`/`compile` need; remove only status/init-specific
  code. If a function is only reachable from a cut command, remove it; if `use`/`compile`
  still calls it, keep it. Verify with a caller search — never guess.
- `http.rs`, `paths.rs`, `logging.rs`, `test_env.rs`, `meta_cmd.rs`, `ops_cmd.rs`,
  `doctor.rs`, `use_command*.rs`, `config.rs`, `help_topics.rs`, `build.rs`, daemon files
  — KEEP.

## Cascade
- Rewrite the `#[command(about = ...)]` tagline — drop "Agent configuration compiler —
  compose, compile, distribute"; describe what `ship` actually is now (an operator + agent
  op-client). Rewrite `AFTER_HELP` to list only surviving commands.
- Remove now-unused deps from `apps/ship-cli/Cargo.toml` (e.g. `rmcp`, `ureq`, possibly
  `reqwest` if no survivor uses it — verify each before removing).
- Delete dead help topics in `help_topics.rs` that describe cut commands.

## Constraints
- Verify-then-delete; never guess. If a "cut" command turns out to be called by a survivor
  or by the dev workflow, STOP and write a `note(out-of-scope)`.
- Pre-launch: delete, no shims, no migration.
- Never push. Commit locally.

## Acceptance
1. `cargo build -p ship-cli` and `cargo build -p shipd` green.
2. `cargo build --workspace --features unstable` / `just check` green.
3. `ship --help` lists ONLY: ops, daemon, doctor, meta, use, compile, config, docs, build
   (+ help). No init/status/validate/install/add/publish/mcp/isolation/run/procs/tail/sui/
   events/audit/workspace/auth/login/logout/whoami/hook.
4. **Dev workflow intact:** `ship use <agent> --path <dir>` still compiles an agent to
   CLAUDE.md (the dispatch.sh path). Spot-check it runs.
5. Surviving ship-cli tests pass; tests for cut commands are removed with their modules.
6. Commit cleanly. DO NOT push.

---
> Execute yourself. Read-only research subagents OK. Commit when acceptance passes. DO NOT push.
