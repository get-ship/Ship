# @ship/authority — Component Spec (ratification)

**Status: ready for code.** 6 components + 2 atoms. Compiled from `.sui`, tokens-only,
attribute-driven. Catalog: `authority.html`. Same conventions as `../chrome/SPEC.md`.
Components with no `<prop>` are **static content** — bind to the projection noted under **Bind**.

The blast/gate vocabulary is two shared atoms (`ship-blast-tag` · `ship-gate-badge`) so the tier and
verdict read identically in op-card, approval-prompt, capability-bar, gate-log.

---

## ship-approval-prompt ★
The Ask-stance checkpoint — a suspended op held at the gate. Clean by default; `open` discloses the call + grant gap + request id. The op never runs before approval.

| prop | type | default | enum | reflect | notes |
|---|---|---|---|---|---|
| `op` | string | `op` | — | — | op name |
| `args` | string | `""` | — | — | disclosed only |
| `req` | string | `""` | — | — | request id (e.g. `ask:mcp.create#7`) — **single token, not `request-id`** |
| `grant` | string | `""` | — | — | the standing-grant gap line |
| `blast` | enum | `C2` | `C0·C1·C2·C3` | ✓ | composes `ship-blast-tag` |
| `state` | enum | `pending` | `pending·approved·denied·expired` | ✓ | pending→verbs · approved→"dispatched" · denied→"never ran" · expired→"released unrun" |
| `open` | bool | `false` | — | ✓ | disclose detail |

- **Parts:** `card·detail`. **Slots:** none.
- **Events:** `ship-approve {req}` · `ship-deny {req}` · `ship-toggle {open}`.
- **Bind:** ▸stream suspend · ▸verb Approve. **Ratify:** state×4 · open vs collapsed · blast×4.

## ship-blast-tag (atom)
An op's blast tier — its inherent authority weight, escalating tone.

| prop | type | default | enum | reflect | notes |
|---|---|---|---|---|---|
| `tier` | enum | `C0` | `C0·C1·C2·C3` | ✓ | C0 muted → C1 info → C2 warning → C3 destructive |

- **Parts:** `tag`. **Ratify:** 4 tiers. **Label policy:** terse `C#` on dense surfaces; word (Read/Write/Elevated/Critical) where the mapping is taught (the capability matrix).

## ship-gate-badge (atom)
A gate decision verdict.

| prop | type | default | enum | reflect | notes |
|---|---|---|---|---|---|
| `decision` | enum | `allowed` | `allowed·approved·needs-approval·denied·running` | ✓ | tone + dot; running pulses |

- **Parts:** `badge`. **Ratify:** 5 decisions.

## ship-approvals-queue
Cross-space inbox for the gate — batches checkpoints with Approve-all / Deny-all.

| prop | type | default | reflect | notes |
|---|---|---|---|---|
| `count` | string | `0` | — | "{count} pending" |

- **Parts:** `queue·body`. **Slots:** default → `ship-approval-prompt`s.
- **Events:** `ship-approve-all · ship-deny-all`. **Bind:** ▸stream suspends across sessions · ▸verb Approve (batched). **Ratify:** N items · empty.

## ship-capability-bar · ship-policy-matrix · ship-principal-list · ship-gate-log  (static content → bind at code time)
| component | role | binds | ratify |
|---|---|---|---|
| `ship-capability-bar` | at-a-glance grants (domain × blast) + denials | ▸read `agent.capabilities` | compact summary; an **editable** variant is the agent-editor's matrix (separate, bespoke picker) |
| `ship-policy-matrix` | full capability × blast grid; granted ≤ tier, locked beyond | ▸read policy | the grid; C3 locked / operator-only |
| `ship-principal-list` | the "who" — principals + grants, edit inline | ▸read principals + policy | rows + max-grant chips |
| `ship-gate-log` | live verdict stream (audit) — composes the atoms | ▸stream gate verdicts | rows × decisions |

### Open items for code
- `ship-capability-bar` editable variant — either a prop (`editable`) or keep the agent-editor's tier-picker as the bespoke editor and the bar as read-only summary. **Decide at code time.**
- The static components (`policy-matrix·principal-list·gate-log`) become prop/row-driven when wired to the live projection; `gate-log` already composes `ship-blast-tag` + `ship-gate-badge`.
