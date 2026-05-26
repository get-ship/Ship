---
group: Architecture
title: Tauri Shell — design
description: A native desktop shell that runs the existing SUI surface as a Connect-Web client over loopback HTTP — same client as browser dev — with a thin Rust core for daemon lifecycle, the updater, and native escalation.
audience: internal
status: design
---

# Tauri shell

Design doc — not yet built. The native desktop app for Ship: a Rust core that
owns the daemon lifecycle and the OS seam, plus a webview that runs the existing
SUI surface as a **Connect-Web client — the same client browser dev uses**. This
is where **Ship-as-its-own-OS becomes a product**: the host adapters, per-agent
identity, and (coming) overlay workspaces surface through one native shell.

## The one rule: Tauri does not replace Connect

Connect is the single wire for **serving and data**. We spent the unification
landing that (one wire, one gate, one contract) and the Tauri shell does not get
to reinvent it. The webview is a Connect-Web client, byte-for-byte the same one
that runs in a browser tab. There is **no IPC data plane**, no `ship_ops(op, json)`
pass-through, and no `tauri-specta` re-typing of the surface. The typed client is
already buf-generated (#79); the shell consumes it unchanged.

IPC is used **selectively**, only for things that are genuinely native and have no
Connect representation:

- **daemon lifecycle** — spawn / supervise / shut down `shipd`,
- **escalated auth** (#86) — the native 2FA / elevation prompt that mints a C3
  session the webview must never hold,
- **updater** — the revived auto-update flow.

That's a handful of hand-typed Tauri commands, not a transport.

## The transport is just HTTP — same as browser dev

The webview reaches the daemon the way it already does in a browser: the
Connect-Web client `fetch`es `http://127.0.0.1:<port>`, the daemon's existing
loopback Connect endpoint. **No custom scheme, no byte proxy, no second
transport.** A custom `ship://`→`ship.sock` proxy was considered and rejected —
it's an IPC data plane wearing a different hat, and re-pipes the exact wire we
just unified. Browser dev and the shell are the *same client over the same
transport*; the only difference is what hosts the webview.

The peer-cred `ship.sock` stays what it is: the **agent** transport (unspoofable
per-agent identity, [agent access](/reference/architecture/agent-access/)) and the
CLI path. The GUI is the **operator**, and loopback HTTP + the operator token is
the established operator path. Don't conflate "agents need the socket" with "the
GUI should too."

## Where the security win actually comes from

Not from a bespoke transport — from things a browser tab can't give you, that the
Tauri webview gets for free:

- **CSP-locked webview, no remote navigation.** A browser tab can be lured to
  `evil.com` and attempt requests at `localhost:<port>`; a Tauri webview pinned to
  the app origin (strict CSP, no remote URLs) cannot be navigated off-origin.
- **native escalation.** C3 (operator/2FA) elevation runs as a native IPC prompt
  (#86); the elevated credential is minted and held by the core, never the webview.

These are posture wins the browser can't match, and none of them require a new
wire.

## Two-token model (where each lives)

The C0–C3 gate is what makes this safe to reason about. The shell splits the
operator's authority across two credentials:

- **Operator token (C0–C2) — lives in the webview.** It's the live operator
  session. Its blast radius is capped at C2 by the gate, so the worst a
  leaked/abused webview session does is dispatch agents and author surfaces —
  never grant capabilities, never touch account / billing / permissions. That
  ceiling is *why* it's tolerable in JS. The persisted-on-disk token stays C0/C1
  (#86); the live webview session is the C2-capable one.
- **Escalated credential (C3) — never in the webview.** A C3 op (e.g.
  `agent.grant`, account/billing/permissions) requires a native step-up:
  1. webview calls `escalate_auth(reason)` IPC,
  2. the Rust core runs the **password / 2FA prompt** (`tauri-plugin-dialog`),
     mints a **short-lived C3 step-up** against the daemon,
  3. the core performs the one C3 operation (or holds the elevated session for a
     tight TTL) and returns only the *result* to the webview.

  The C3 credential never crosses into JS. This keeps the tier boundary
  load-bearing: webview compromise tops out at C2; crossing to C3 always demands a
  live human gate.

**Caveat — third-party views.** A webview-resident operator token is readable by
*any* JS in that webview. That's fine for first-party SUI views. Once
untrusted/third-party code can render, the token story changes. **How third-party
code is confined and brokered is an open design conversation** — it is deliberately
not settled here. See `.artifacts/discuss/third-party-code.md`.

## The ownership seam

Tauri is Rust-core + webview. That maps onto the existing split:

- **Rust core — ours.** Daemon lifecycle (spawn/supervise `shipd`), the native C3
  step-up (password/2FA + elevated credential), the `ship-host` adapters (cgroup /
  peer-cred / overlay), and the selective native IPC commands (escalation,
  lifecycle, updater).
- **Webview — the frontend/other agent's lane.** The SUI views + the Connect-Web
  client over loopback HTTP. The shell hosts it; it doesn't author it or re-type it.

## Architecture (target)

```
┌─ Tauri app ──────────────────────────────────────────┐
│  webview: SUI views + Connect-Web client (unchanged)  │  ← frontend lane
│   holds operator token (C0–C2, capped by the gate)    │
│        │                                               │
│        │  fetch() Connect over http://127.0.0.1:<port> │
│        │  (Authorization: operator token)              │
│  Rust core (ours):                                     │
│    • daemon lifecycle (spawn/supervise shipd)          │
│    • escalate_auth: password/2FA → C3 step-up (held    │
│      by the core, never the webview)                   │
│    • native IPC: updater / lifecycle                   │
│    • ship-host adapters (cgroup/overlay)               │
└────────────────────────────────────────────────────┬─┘
                                                       ▼
                                       shipd Connect over loopback
                                       (operator token C0–C2 / C3 step-up)
```

One wire (Connect over HTTP), one gate, one contract — the same client browser dev
uses. The webview holds only the C2-capped operator token; C3 lives behind the
native gate; the shell never re-implements the surface.

## Mapping to Tauri's own security model

Tauri v2 ships its own access-control layer. It does **not** replace our
blast-tier gate — it sits in a different place, and conflating them is a mistake.
Two distinct planes:

- **Data plane (Connect over loopback `fetch`).** Gated by (a) the **CSP**
  `connect-src` allowlist, (b) the daemon's **service×blast-tier gate**, (c) the
  operator token. Tauri's capability ACL and the Isolation pattern **do not touch
  `fetch`** — they only intercept Tauri IPC. So the data plane's authority is
  entirely Ship's existing gate; Tauri adds only the origin/CSP fence around it.
- **Native IPC plane (our few commands + plugin commands).** Gated by Tauri
  **capabilities** (`src-tauri/capabilities/*.json`: which window may invoke which
  permission) and optionally the **Isolation pattern** (a sandboxed iframe that
  intercepts every IPC payload before it reaches Rust, per-run AES-GCM keyed).

Concrete decisions for the spike:

- **CSP** (`tauri.conf.json` → `app.security.csp`), locked to loopback + IPC:

  ```
  default-src 'self';
  connect-src 'self' ipc: http://ipc.localhost http://127.0.0.1:<port>;
  script-src 'self'; style-src 'self' 'unsafe-inline'
  ```

  Tauri hashes local scripts and nonces styles/external scripts at build time; we
  only enumerate trusted hosts. No remote origins — this is the "can't be lured to
  evil.com" fence.
- **Capabilities.** One capability scoped to the `main` window granting exactly:
  the updater (`updater:default`), dialog (`dialog:default` — the 2FA prompt),
  process/lifecycle, and our custom `escalate_auth` / daemon-lifecycle commands.
  Nothing wildcard.
- **Isolation pattern** is the right chokepoint for `escalate_auth` — a minimal,
  dependency-free isolation app can validate the IPC payload (e.g. the requested
  C3 op is in an allowed set) before it reaches the core. Keep it tiny to avoid
  making the security layer itself an attack surface.

### Third-party code confinement — open conversation, not decided here

How Ship runs code it didn't write — third-party views, pipeline/trigger/hook
logic, integrations — is an **active design conversation**, not a settled part of
this shell doc. The shell's job is narrow: host the first-party SUI surface as a
Connect client and own the native seam. The confinement/brokering model
(capability brokering vs confinement cells, `fetch`-as-platform-API, the
pipeline/hook sandbox engine, integration creds-behind-Ship) is tracked separately
in `.artifacts/discuss/third-party-code.md` until it converges. Nothing about
multi-pane origin isolation, SES endowment policy, or a sandbox matrix is asserted
here — those were earlier drafts and have been moved to that discussion.

One mechanical fact this doc *does* fix, because it constrains the shell: Tauri's
capability ACL and Isolation pattern gate **IPC**, not webview `fetch`. So whatever
the third-party model becomes, it cannot lean on Tauri Isolation to confine the
Connect data plane — that confinement has to come from Ship's own gate/brokering.

## Sequencing — why this waits on Connect unification

The shell pays off **after** the GUI's REST→Connect flip (#72, the other agent's
in-flight work). The webview must be a Connect-Web client before the shell hosts
it; otherwise the shell is just a wrapper around the REST problem. Order:

1. **(landed)** loopback Connect endpoint + service×blast-tier gate — the surface
   the webview targets.
2. **(other agent, #72)** GUI fetches/streams REST → Connect; retire REST/MCP.
3. **Tauri core spike** — daemon lifecycle + CSP-locked webview + operator-token
   injection. The webview must be Connect-only first.
4. **Bundling + capability-approval UX** — the install/grant flow, app packaging,
   the revived updater.
5. **Overlay + isolation** surface through the same core (`ship-host`
   provision_workspace from [agent workspaces](/reference/architecture/agent-workspaces/)).

## Open decisions (for the spike)

- **Daemon: embed vs spawn.** Spawn the existing `shipd` binary and supervise it
  vs run it in-process. Leaning **spawn + supervise** — one daemon implementation,
  matches the CLI's lifecycle, the shell is just another (operator) client.
- **Token injection mechanism.** The core holds the operator token and must attach
  it to webview requests without exposing it to JS — Tauri request interception /
  a per-session header. Confirm Tauri v2 can rewrite outbound webview request
  headers; if not, fall back to a one-time `__TAURI__`-gated handshake that hands
  the webview a short-lived scoped token.
- **CSP lockdown.** Strict `connect-src 'self' http://127.0.0.1:<port>`, no remote
  origins, no `unsafe-eval`. This is the actual posture boundary — get it right.
- **Auth posture.** The shell is the operator. Escalation to C3 goes through the
  native `escalate_auth` IPC (#86); the elevated session is held by the core, never
  the webview.
- **Per-OS.** Daemon-launch + loopback binding differ per OS behind the same
  `ship-host` seam.
