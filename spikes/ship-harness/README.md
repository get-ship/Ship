# ship-harness-spike

Phase-1 proof of concept for a **first-party Rust coding agent for Ship**, built
by cobbling: **rig** (substrate) + **claw-code** (harness shapes, reference) +
**oh-my-pi** (native muscle, to lift later). MIT all round.

It exists to prove three load-bearing claims from the design brief, end-to-end,
against the *real* rig checkout at `/home/madvib/dev/rig`:

1. **Substrate** — rig drives the agent loop for **Anthropic** and any
   **OpenAI-compatible** endpoint. One `drive<M: CompletionModel>()`, two providers.
2. **Verification spine** — *nothing gets blindly accepted*. Every tool call runs
   `Permit → Execute → Verify` (`src/spine.rs`). `edit_file` re-reads from disk to
   confirm the edit landed; `bash` surfaces the real exit code and refuses nonzero.
   A failed verify is returned to the model as `UNVERIFIED`, never folded into context.
3. **Two-rail tool model** (`src/rails.rs`) — forced by Ship's actual shape:
   - Rail A: structured/daemon work → `ship ops call <op>` (gated by Ship's
     capability + blast-radius at the peer-cred socket). The harness does **not**
     reimplement git — `ship_op` routes to it.
   - Rail B: shell/exec → the **sandbox** (sidecar `Processes/Run` inside a Ship
     isolation tier). The default `Sandbox` **refuses host exec by construction**
     — "bash on the host is insanity."

## Map to the design

| Spike piece | Production target |
|---|---|
| `auth.rs` `AuthContext` | mirror of `apps/shipd/src/auth/mod.rs:73`; **constructed in-process** = third provenance beside peer-cred + token. `operator_cred=false` ⇒ never clears C3 |
| `transport.rs` `OpsTransport` (InProcess) | direct in-process call — no socket, no PID. The base Ship agent |
| `transport.rs` `Gateway` seam | shipd impl wraps `services::command::call` (the one gate). **Untouched while ops SSOT is mid-refactor** |
| `transport.rs` `SocketTransport` | out-of-process agents (external harness / Ship Code sidecar), SO_PEERCRED attested |
| `spine.rs` `Blast` C0–C3 | `proto/ship/ops/v1/policy.proto` blast tiers; permit gate defers to `services/gate.rs` |
| `rails.rs` `Sandbox` | sidecar `ship.sidecar.v1.Processes/Run` inside `ship-isolation` tier |
| `RefuseHostSandbox` | the real default — harness never holds host-exec authority |
| rig `Agent`/`Tool` loop | the harness, dispatched by `shipd` with `harness:"ship"` |

**Auth note:** identity is no longer tied to a connecting process PID. That's the
Ship-shell AI unlock — an embedded agent can live inside the shell/daemon as a
first-class surface rather than a terminal bolted to a window's PID.

## Run

```sh
# No API key, no tokens — deterministic drive of spine + rails. Best first look.
SHIP_PROVIDER=selftest cargo run
SHIP_PROVIDER=selftest SHIP_SPIKE_ALLOW_HOST_BASH=1 cargo run   # see bash + the nonzero-exit refusal

# Anthropic
SHIP_PROVIDER=anthropic ANTHROPIC_API_KEY=... cargo run -- "read Cargo.toml and summarize"

# OpenAI-compatible (Ollama / vLLM / OpenRouter / ...)
SHIP_PROVIDER=openai-compat OPENAI_API_KEY=... \
  SHIP_BASE_URL=http://localhost:11434/v1 SHIP_MODEL=llama3.1 cargo run -- "..."

# bash is inert until a sandbox is attached. Opt into host exec for LOCAL
# exercise only (stands in for the sidecar client; never ships):
SHIP_SPIKE_ALLOW_HOST_BASH=1 SHIP_PROVIDER=anthropic ANTHROPIC_API_KEY=... cargo run
```

## What this spike deliberately does NOT do (next phases)

- Port claw-code's `ConversationRuntime`/permission-enforcer/JSONL-session shapes
  onto these traits (rig gives the loop; claw gives the harness ergonomics).
- Lift oh-my-pi `pi-shell` (minimizer), `pi-iso` (cheap clone + **diff** → feeds
  the verify gate), `pi-ast`, in-process `grep`.
- Real `ship ops` transport + SO_PEERCRED identity + event emission.
- Escalate verify from deterministic probes to adversarial sub-agent skeptics.
