# @ship/chrome — Component Spec (ratification)

**Status: ready for code.** 7 components. Compiled from `.sui`, tokens-only, attribute-driven
(presentation has no controller — state is reflected-attribute CSS). Production wires the
**Events** + **Bind contract** noted per component. Catalog: `chrome.html` renders every state.

Conventions: **reflect** = the prop mirrors to an attribute for `:host([x])` styling. `::part`s are
the styling handles exposed to the host. Events are `CustomEvent`s the controller should emit/accept.
Multi-word prop values are single tokens (no hyphenated accessors — they break the compiler).

---

## ship-command-bar ★
The cmdk keystone — summoned palette (⌘K), opens over any surface, never docked. `ai` mode hosts agent chat.

| prop | type | default | enum | reflect | notes |
|---|---|---|---|---|---|
| `open` | bool | `true` | — | ✓ | `:host(:not([open]))` → `display:none` |
| `mode` | enum | `find` | `find · search · create · ai` | ✓ | drives placeholder hint + active mode tab |

- **Slots:** default → results list (find/search/create) or chat turns (ai: `sui-message` · `ship-op-card` · `ship-approval-prompt`).
- **Parts:** `bar` · `input` · `body`.
- **Events:** `ship-open` / `ship-close` (toggle, from ⌘K / esc / scrim) · `ship-mode-change {mode}` · `ship-submit {query}` (↵) · `ship-select {id}` (result chosen).
- **Bind:** ai mode ▸stream a session · ▸verb Steer. Reconciles reference `sui-command-chat`.
- **Ratify states:** open×{find, search, create, ai} (4) · closed (1) · ai with live turns vs empty.

## ship-activity-bar
Vertical context switcher — each item **is a context** (the configurable domains of a space). Not a generic "Explorer"; Library is not here.

| prop | type | default | enum | reflect | notes |
|---|---|---|---|---|---|
| `active` | enum | `views` | `agents · views · components · pipelines · themes · collections · artifacts · settings` | ✓ | active item → primary + accent bar; settings pinned to foot |

- **Slots:** none (baked icon set). **Parts:** none yet — add `item` if hosts need per-item styling.
- **Events:** `ship-context-change {context}` (item click).
- **Bind:** selection drives the navigator panel (tree vs list) — host concern.
- **Ratify states:** 8 (one per `active` value) — verify the accent bar + primary tint land on the right item.

## ship-file-tree
Titled navigator panel holding `ship-tree-item`s for ONE file-backed context. Header label = the context name.

| prop | type | default | enum | reflect | notes |
|---|---|---|---|---|---|
| `label` | string | `""` | — | — | context name (e.g. "Views") — not "Explorer" |
| `scope` | string | `""` | — | — | e.g. `.ship/` or `~/.ship/` |

- **Slots:** default → `ship-tree-item` rows. **Parts:** `head` · `body`.
- **Events:** `ship-new` (＋) · `ship-collapse-all`.
- **Ratify states:** with items (nested) · empty · long (scroll).

## ship-tree-item
One file-tree row — folder (toggles) or file (with filetype tag + status dot).

| prop | type | default | enum | reflect | notes |
|---|---|---|---|---|---|
| `name` | string | `""` | — | — | row label |
| `kind` | enum | `file` | `folder · file` | ✓ | folder shows twisty; file hides it, shows glyph off |
| `filetype` | enum | `none` | `sui · ts · css · json · md · none` | ✓ | colored tag; `none` hides tag |
| `open` | bool | `false` | — | ✓ | folder twisty rotation |
| `selected` | bool | `false` | — | ✓ | primary tint + bold |
| `depth` | number | `0` | — | — | indent = `8px + depth·14px` |
| `status` | enum | `none` | `none · dirty · run` | ✓ | dot: dirty=warning, run=success |

- **Slots:** none. **Parts:** `row`.
- **Events:** `ship-toggle {open}` (folder) · `ship-select` (file).
- **Ratify states:** folder open/closed · file × 5 filetypes · selected · status dirty/run · depth 0–3.

## ship-status-bar
Bottom strip — **glance-only Ship-native ambient state**. Deliberately NOT VSCode: no git, no problems, no Ln/Col.

| prop | type | default | enum | reflect | notes |
|---|---|---|---|---|---|
| `daemon` | enum | `ready` | `ready · busy · offline` | ✓ | health dot: ready=success, busy=warning+pulse, offline=dim |
| `sessions` | string | `0` | — | — | live agent sessions; emphasized in accent |
| `approvals` | string | `0` | — | ✓ | pending gate; **`[approvals='0']` hides the segment** |
| `sync` | enum | `synced` | `synced · syncing · offline` | ✓ | syncing spins the icon |
| `space` | string | `""` | — | — | active space name |
| `context` | string | `""` | — | — | freeform right-of-center context string |
| `theme` | string | `Ship Light` | — | — | current theme label |

- **Slots:** none. **Parts:** none.
- **Events:** `ship-open-runs` (click sessions/approvals → opens the run/session view).
- **Ratify states:** daemon×3 · sync×3 · approvals 0 (hidden) vs >0 · busy pulse.

## ship-dock
Floating blurred desktop tray — apps + divider + Ship surfaces. The shell launcher.

| prop | type | default | enum | reflect | notes |
|---|---|---|---|---|---|
| — | — | — | — | — | no props; pure container |

- **Slots:** default → `ship-dock-app` tiles + `.div` dividers. **Parts:** none.
- **Ratify states:** N apps + divider; with an active app.

## ship-dock-app
One dock tile — icon-only launcher, active dot below.

| prop | type | default | enum | reflect | notes |
|---|---|---|---|---|---|
| `label` | string | `""` | — | — | tooltip |
| `active` | bool | `false` | — | ✓ | shows the active dot |
| `hue` | string (css color) | `""` | — | — | icon color |

- **Slots:** default → an `<svg>` or `sui-icon`. **Parts:** `app`.
- **Events:** `ship-launch {label}`.
- **Ratify states:** default · hover (lift) · active.

---

### Open items for code
- **`::part` coverage** — add `item` to `ship-activity-bar` and a per-row part to `ship-file-tree` if hosts need to style individual entries.
- **Events are intended, not yet emitted** — the `.sui` here is presentation-only; the controller (`.controller.ts`) wires the CustomEvents above.
- **Icon source** — baked inline SVGs match the reference; swap to `sui-icon` (lucide registry) at code time for consistency with the rest of the system.
