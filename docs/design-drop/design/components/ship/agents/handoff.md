# @ship/agents + @ship/authority — component handoff

**Status: design-complete, ready to implement.**
Source: `design/components/ship/{agents,authority}/*.sui` — tokens-only, attribute-driven,
no controller needed for the visual (states are reflected-attribute selectors). Production
wires the controllers + the bindings noted below. Compiled live via `tools/catalog/sui-workbench.js`.

## Conventions
- **Tokens only** (`var(--token)` from `sui/ship.css`). No hex.
- **State = reflected prop** → `:host([state=…])` styling. No JS for presentation.
- **Binds:** `▸stream` = WatchSession subscription · `▸read` = projection read · `▸verb` = control RPC.
- Multi-word props are **single tokens** (`req`, not `request-id`) — hyphenated accessors break the compiler.

## @ship/agents

| tag | role | props / states | binds | composes |
|---|---|---|---|---|
| `ship-session-list` | sessions in a Space; status fold | rows: active·suspended·completed·failed·cancelled; status-filter header | ▸read `sessions` · ▸stream status | status-dot, harness badge |
| `ship-session-metrics` | session rollup strip | `duration·turns·tools·errors·cost·outcome(completed\|failed\|cancelled)` | ▸read sessions + analytics | — |
| `ship-op-card` | a gated op ★ | `op·args·result·reason·ms` · `blast(C0–C3)` · `decision(allowed\|approved\|needs-approval\|denied\|running)` | ▸stream op events | **ship-blast-tag · ship-gate-badge** |
| `ship-agent-controls` | the verbs, context-aware | `agent` · `state(idle\|active\|suspended\|completed\|failed)` → shows Start/Steer/Approve/Deny/Stop | ▸verb Start·Steer·Approve·Stop | — |
| `ship-usage-chip` | tokens + cost | `input·output·reasoning·cache·cost` | ▸read analytics_turns | — |
| `ship-agent-card` | browse / launch a def | identity·harness·capability summary·Start·status | ▸read AgentDefs · ▸verb Start | harness badge, capability chips |
| `ship-agent-chat` | the thread, presentation-agnostic | **`surface(cmdk\|drawer)`** · `agent·session` · `state(idle\|streaming\|awaiting)`; turns **slotted** | ▸stream session · ▸verb Steer | sui-message, op-card, approval-prompt, composer. **Reconciles `sui-command-chat` (ai mode).** |
| `ship-agent-canvas` | the **Agent Canvas** feature (chrome) | `agent·session` · `activity(idle\|thinking\|morphing)`; controls: undo·stop·save; slots **agent-slots** | ▸stream render ops · ▸verb Stop·Save | ship-agent-slot |
| `ship-agent-slot` | a region an agent injects into ★ primitive | `state(empty\|rendering\|filled\|saved)` · `label`; content **slotted** | ▸stream morph ops · ▸verb Save | anything (general) |

**Agent Canvas** is session-scoped, scratch by default, launched from the persistent chat.
`ship-agent-slot` is general — agent-canvas is one composition that tiles them; usable in a view, card, or drawer too.

## @ship/authority

| tag | role | props / states | binds |
|---|---|---|---|
| `ship-approval-prompt` | Ask-stance checkpoint ★ | `op·args·req·grant` · `blast(C0–C3)` · `state(pending\|approved\|denied\|expired)` | ▸stream suspend · ▸verb Approve |
| `ship-capability-bar` | standing grants (domain×blast) + denials | static from `agent.capabilities` | ▸read agent.capabilities |
| `ship-blast-tag` | blast tier atom | `tier(C0\|C1\|C2\|C3)` | — |
| `ship-gate-badge` | gate decision atom | `decision(allowed\|approved\|needs-approval\|denied\|running)` | — |

`blast-tag` + `gate-badge` are extracted from `op-card` so the verdict reads identically in op-card,
approval-prompt and capability-bar. (op-card composes them at code time.)

## Surfaces (compositions — NOT components)
Same parts, different surfaces — this is the point of the pack:
- **Session workspace** = agent-controls header + transcript (message + op-card) + approval + metrics.
- **Agent Canvas** (interactive chat) = agent-canvas + agent-slots; chat via agent-chat (`surface` setting).
- **Pipeline review** = approval-prompt + op-card in a review queue (different surface, same components).
- **cmdk / drawer chat** = agent-chat `surface` prop; both toggle/hide → full canvas.

## Decisions (locked)
- **Save** = scratch → artifact (md/sui/image). "Save as view" = future graduation.
- **Canvas** = session-scoped chrome, launches from the persistent chat.
- **Chat presentation** = `surface` prop (cmdk \| drawer), a user setting.
- **Prefix** = generality: `ship-*` here (runtime-bound); `sui-*` reserved for framework-general atoms.

## Authority pack — expanded (build 2)
Substantive authority components beyond the original two:
- `ship-approvals-queue` — cross-space inbox; batches checkpoints (composes `approval-prompt`); `count`; Approve-all / Deny-all. ▸stream suspends · ▸verb Approve(batch).
- `ship-policy-matrix` — capability × blast grid; granted ≤ tier, locked beyond. ▸read policy.
- `ship-principal-list` — the "who"; principals + grants, edit inline. ▸read principals + policy.
- `ship-gate-log` — live verdict stream (audit); composes the atoms. ▸stream gate verdicts.

→ Authority is now **6 components + 2 atoms**. Catalog: `authority/authority.html`.

## Progressive disclosure (build 2)
`ship-op-card` and `ship-approval-prompt` gained an **`open`** reflected-attr: clean by default
(op-card folds `args`; approval collapses to the ask + the verbs), detail on disclose. Production
wires the chevron click; the design renders both states via the attribute. Extend the same pattern
to the in-chat op-cards.
