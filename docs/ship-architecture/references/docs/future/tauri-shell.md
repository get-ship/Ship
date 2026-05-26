---
group: Architecture
title: Tauri Shell — design
description: A native desktop shell whose Rust core embeds the daemon and bridges the webview to Connect — dissolving the browser↔REST problem and giving Ship-as-OS a product surface.
audience: internal
status: design
---

# Tauri shell

Design doc — not yet built. The native desktop app for Ship: a Rust core that
owns the daemon lifecycle and the OS seam, plus a webview that runs the existing
SUI surface. This is where **Ship-as-its-own-OS becomes a product** — the host
adapters, per-agent identity, and (coming) overlay workspaces surface to the user
through one native shell.

## Why Tauri, and what it dissolves

The browser GUI today is a **cross-origin REST client** of `shipd` — the thing the
[op-surface](/reference/architecture/op-surface/) migration is retiring. Tauri
removes that problem structurally: the webview talks to the **Rust core via IPC**,
and the Rust core speaks **Connect over the host socket** (`ship.sock`, the same
per-agent-auth transport from [agent access](/reference/architecture/agent-access/)).
No browser, no cross-origin REST, no token-in-URL — the shell is a first-class
local client, not a web page reaching into a daemon.

It's also the natural home for: the **daemon lifecycle** (launch/embed/supervise/
shutdown), the **capability-approval UX** (the `agent.grant` / install-time
elevation flow from the [auth model](/reference/architecture/auth-model/)), and
**bundling/distribution**.

## The ownership seam (this is the "us first" part)

Tauri is Rust-core + webview. That maps cleanly onto the existing split:

- **Rust core — ours.** Daemon lifecycle, the `ship-host` adapters (cgroup /
  peer-cred / overlay), and the **IPC↔Connect bridge**: a Tauri command layer
  that forwards webview calls to `shipd` over `ship.sock` and streams events back.
  This bridge *is* the thing that dissolves browser-REST.
- **Webview — the frontend/other agent's lane.** The SUI views + the Connect-Web
  TS client, rendered in the webview. The shell hosts it; it doesn't author it.

The boundary is the IPC/Connect bridge. We own the native core + how the app
bundles and launches the daemon; they own the view layer. No splintering.

## Architecture (target)

```
┌─ Tauri app ──────────────────────────────────────────┐
│  webview (SUI views + Connect-Web client)             │  ← frontend lane
│        │  Tauri IPC (commands + event channel)         │
│  Rust core (ours):                                     │
│    • daemon lifecycle (spawn/embed/supervise shipd)    │
│    • IPC ↔ Connect bridge  ──────────────┐             │
│    • ship-host adapters (cgroup/overlay) │             │
└──────────────────────────────────────────┼───────────┘
                                            ▼
                                   shipd over ship.sock
                              (peer-cred auth, the op surface)
```

The bridge calls the **same Connect op surface** agents use — so there's one wire
(Connect), one gate, one contract. The webview never sees a token or a port.

## Sequencing — why this waits on Connect unification

The Tauri core pays off **after** the GUI's REST→Connect flip (the other agent's
in-flight work). If the webview still speaks REST, the Rust core would have to
bridge REST too, defeating the point. Order:

1. **(landed)** per-agent socket auth + service×blast-tier gate (this is the wire
   the bridge targets).
2. **(other agent)** GUI fetches/streams REST → Connect; retire REST/MCP.
3. **Tauri core spike** — daemon lifecycle + IPC↔Connect bridge on the host socket.
   Can scaffold in parallel now (the bridge targets the already-shipped socket),
   but wire the webview to Connect-only.
4. **Bundling + capability-approval UX** — the install/grant flow, app packaging.
5. **Overlay + isolation** surface through the same core (the `ship-host`
   provision_workspace from [agent workspaces](/reference/architecture/agent-workspaces/)).

## Open decisions (for the spike)

- **Daemon: embed vs spawn.** Run `shipd` in-process (a Tauri-managed thread/
  runtime) vs spawn the existing binary and supervise it. Spawn keeps one daemon
  implementation + matches the CLI's lifecycle; embed avoids a second process.
  Leaning **spawn + supervise** (one shipd, the shell is just another client).
- **Bridge shape.** Thin generic pass-through (one `ship_ops(op, json)` Tauri
  command + an event-stream channel) vs typed per-op commands. Leaning the **thin
  pass-through** — the webview already has the typed Connect-Web client; the bridge
  is transport, not re-typed surface.
- **Auth on the bridge.** The shell is the operator — its bridge connection to
  `ship.sock` resolves as operator (same-uid peer-cred, no agent container). No
  token handling in the webview. Confirm this is the desired posture (operator-
  trust for the local shell) vs a scoped session.
- **Per-OS.** Tauri is cross-platform; the daemon-launch + socket path differ per
  OS and sit behind the same `ship-host` seam (Windows: named pipe, not `ship.sock`
  — the named-pipe transport that's already the Windows follow-up).
