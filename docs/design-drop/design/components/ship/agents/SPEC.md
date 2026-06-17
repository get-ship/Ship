# @ship/agents — Component Spec (ratification)

**Status: ready for code.** 9 components. Compiled from `.sui`, tokens-only, attribute-driven.
Catalog: `agents-pack.html`. Companion: `handoff.md` (roles + composition). Same conventions as
`../chrome/SPEC.md` (reflect / parts / events / bind). Components with no `<prop>` are **static
content** here — at code time they bind to the projection noted under **Bind**.

---

## ship-op-card ★
One gated op (tool call) — authority made visible. Clean by default; `open` discloses args.

| prop | type | default | enum | reflect | notes |
|---|---|---|---|---|---|
| `op` | string | `op` | — | — | op name (e.g. `collections.read`) |
| `args` | string | `""` | — | — | folded behind the chevron |
| `result` | string | `""` | — | — | shown inline on success |
| `reason` | string | `""` | — | — | denial reason (decision=denied) |
| `ms` | string | `""` | — | — | duration suffix (e.g. `· 40ms`) |
| `blast` | enum | `C0` | `C0·C1·C2·C3` | ✓ | composes `ship-blast-tag` at code time |
| `decision` | enum | `allowed` | `allowed·approved·needs-approval·denied·running` | ✓ | composes `ship-gate-badge` |
| `open` | bool | `false` | — | ✓ | disclose the dense args block |

- **Parts:** `card`. **Slots:** none. **Events:** `ship-toggle {open}` (chevron).
- **Bind:** ▸stream op events. **Ratify:** decision×5 × blast×4 (focus C0-allowed, C2-needs-approval, C2-denied, C3-running) · open vs collapsed.

## ship-approval-prompt → see ../authority/SPEC.md
(authority pack; listed here because the transcript composes it inline.)

## ship-agent-controls
Context-aware verbs — only the valid actions for a state render.

| prop | type | default | enum | reflect | notes |
|---|---|---|---|---|---|
| `agent` | string | `""` | — | — | label |
| `state` | enum | `active` | `idle·active·suspended·completed·failed` | ✓ | idle→Start · active→Steer/Stop · suspended→Approve/Deny/Stop · completed/failed→Rerun |

- **Parts:** none. **Slots:** none. **Events:** `ship-start·ship-steer·ship-approve·ship-deny·ship-stop·ship-rerun`.
- **Bind:** ▸verb the matching control RPC. **Ratify:** 5 states (verify which verbs show).

## ship-usage-chip
Token + cost readout — turn footer / detail header / dashboard.

| prop | type | default | reflect | notes |
|---|---|---|---|---|
| `input·output` | string | `0` | — | token counts |
| `reasoning·cache` | string | `""` | — | optional; hidden when empty |
| `cost` | string | `""` | — | e.g. `$0.04` |

- **Parts:** none. **Slots:** none. **Bind:** ▸read `analytics_turns`. **Ratify:** full vs minimal (no reasoning/cache).

## ship-session-metrics
Session rollup strip.

| prop | type | default | enum | reflect | notes |
|---|---|---|---|---|---|
| `duration` | string | `—` | — | — | e.g. `32s` |
| `turns·tools·errors` | string | `0` | — | — | counts |
| `cost` | string | `—` | — | — | total |
| `outcome` | enum | `completed` | `completed·failed·cancelled` | ✓ | tones the strip |

- **Bind:** ▸read sessions + analytics. **Ratify:** outcome×3.

## ship-agent-chat
The thread, presentation-agnostic — `surface` is a user setting; both hide to reveal a full canvas.

| prop | type | default | enum | reflect | notes |
|---|---|---|---|---|---|
| `surface` | enum | `cmdk` | `cmdk·drawer` | ✓ | cmdk=centered palette · drawer=edge sheet |
| `agent` | string | `agent` | — | — | header identity |
| `session` | string | `""` | — | — | sub-label; the Switch picker target |
| `state` | enum | `idle` | `idle·streaming·awaiting` | ✓ | header dot color |

- **Parts:** `header·body`. **Slots:** default → turns (`sui-message`·`ship-op-card`·`ship-approval-prompt`).
- **Events:** `ship-steer {text}` (composer submit) · `ship-switch-session`.
- **Bind:** ▸stream session. Reconciles `sui-command-chat`. **Ratify:** surface×2 × state×3.

## ship-agent-canvas
The Agent Canvas (feature) — session-scoped chrome over a field of slots. Scratch by default.

| prop | type | default | enum | reflect | notes |
|---|---|---|---|---|---|
| `agent` | string | `agent` | — | — | control-bar identity |
| `session` | string | `""` | — | — | sub-label |
| `activity` | enum | `idle` | `idle·thinking·morphing` | ✓ | "what's happening" indicator |

- **Parts:** `controls·field`. **Slots:** default → `ship-agent-slot`s.
- **Events:** `ship-stop·ship-save·ship-undo`. **Bind:** ▸stream render ops · ▸verb Stop·Save. **Ratify:** activity×3.

## ship-agent-slot ★ (primitive)
A region an agent injects into — general, not coupled to the canvas.

| prop | type | default | enum | reflect | notes |
|---|---|---|---|---|---|
| `state` | enum | `filled` | `empty·rendering·filled·saved` | ✓ | empty=awaiting · rendering=skeleton · filled=interactive (Save on hover) · saved |
| `label` | string | `""` | — | — | empty-state prompt |

- **Parts:** `slot·content`. **Slots:** default → injected content.
- **Events:** `ship-save`. **Bind:** ▸stream morph ops · ▸verb Save. **Ratify:** 4 states.

## ship-session-list · ship-agent-card  (static content → bind at code time)
| component | binds | ratify |
|---|---|---|
| `ship-session-list` | ▸read `sessions` · ▸stream status | the 5-status fold (active·suspended·completed·failed·cancelled) + status filter |
| `ship-agent-card` | ▸read AgentDefs · ▸verb Start | identity · harness · capability summary · Start; status chip |

### Open items for code
- `ship-op-card` / `ship-approval-prompt` should **compose** `ship-blast-tag` + `ship-gate-badge` (currently inline markup) — extract at code time so the verdict is one atom everywhere.
- Events listed are intended; the `.controller.ts` emits them.
- `ship-session-list` / `ship-agent-card` rows: turn the static fixtures into prop-driven row sub-components (`ship-session-row`) when wired to the live projection.
