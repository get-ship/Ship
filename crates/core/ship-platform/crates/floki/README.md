# floki

**Layer:** L1 — **Yoke** (crate still named `floki`): embeddable native agent harness.
**Floki** is reserved as a default agent *profile* / future model-family name, not this crate.

## Providers (no `SHIP_*` selection)

| Provider | `YOKE_PROVIDER` | API key env | Base URL |
|----------|-----------------|-------------|----------|
| Anthropic | `anthropic` | `ANTHROPIC_API_KEY` | n/a |
| OpenAI | `openai` | `OPENAI_API_KEY` | optional `OPENAI_BASE_URL` |
| **NVIDIA Build / Integrate** | `nvidia` | **`NVIDIA_API_KEY`** | optional `NVIDIA_BASE_URL` (default Integrate `/v1`) |
| Generic OpenAI-wire | `openai-compat` | `OPENAI_COMPAT_API_KEY` | required `OPENAI_COMPAT_BASE_URL` |

Selection: `YOKE_PROVIDER` + `YOKE_MODEL`. Never put an NVIDIA key in `OPENAI_API_KEY`.

### Smoke (no Ship)

```bash
export YOKE_PROVIDER=nvidia
export NVIDIA_API_KEY=nvapi-...
export YOKE_MODEL=<catalog-model-id>
cargo run -p floki --bin yoke-smoke
```

Floki is the runtime where Ship owns the agent loop. Because the loop is ours, every effect enters at the single gated door (`gateway::Gateway`); code-mode JS runs in a locked-down V8 isolate (`ship-js`) whose only effect path is the same gate.

## Key seams

- `gateway` — `Gateway`, `InProcessGateway`, `GatewayCallKind`: the ONE effect door. All ops — tool calls from the model and SDK calls from JS scripts — land here.
- `gate` — grant-list enforcement; projects `agent_def::Agent::capabilities` into what the gateway permits.
- `agent_loop` — `run_step` drives one model turn using the commodity rig. Two tools: `ship_op` (single gated op) and `run_script` (code-mode JS).
- `dispatch` — `GatewayDispatch` adapts `Gateway` to the `ship_js::Dispatch` seam so the JS VM and the durable-execution engine share ONE op contract.
- `durable` — `run_durable_script`: a ship-js script run over `durable_execution::JournalingDispatch` (Stage 1 — journal + replay for code-mode; live-loop durability is Stage 2, tracked in #30).
- `session` *(internal, `pub(crate)`)* — `HistoryStore`/`Session`/`Turn` history primitives kept in-crate ahead of consumers. Not part of the public surface; the live durable-conversation path is `durable_loop`.
- `spine` — post-exec verify + `Ask` approval handshake.
- `provider` — model construction and credential resolution.
- `sdkgen` — per-session SDK generation (the soft layer; the gate is the hard one).

## Agent definition

Agent definitions are read from `agent_def` — the shared type both Floki and ship-code read. Floki does not duplicate the schema.
