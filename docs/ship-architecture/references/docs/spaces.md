---
group: Architecture
order: 6
title: Spaces
description: What a space is, how one daemon serves many, and how the daemon stays authoritative over space windows.
audience: internal
---

# Spaces

A **space** is the unit of isolation, coordination, and config. It is sandboxed,
launches independently in its own window, and contains the things an agent works
with — views, agents, skills, pipelines, triggers, collection-schemas. Agents
coordinate *within* a space and do not cross between spaces. A space is one
`.ship/` directory; `git` is just one of its views. (Spaces can be synced and
shared, and can depend on store apps — both future; this doc describes what
stands today.)

## One daemon, many spaces

There is exactly **one `shipd`**. It is not forked per space. Its local axum
server serves the space webviews — a handful of concurrent single-user apps,
same-origin. A space identity is a path-hash of the space root. There is **no
daemon home space**: a discoverable project root becomes the active space, but
with no discoverable project the daemon boots **space-less**. The **emit path
resolves the active space per-connection**, not from a process global
(`ship_substrate::project::resolve_active_space` is the per-request seam).

- `DaemonSession.space_id` and `AuthContext.space_id` carry the requesting
  connection's space. A space window's session carries *its* space.
- `AuthContext::to_emit_context()` stamps `space_id` from the session. A
  connection with no resolved space keys `None` (operator/CLI) — there is no
  home/global space to fall back onto.
- So an event a window emits is partitioned into *that window's* space, and the
  event log is space-partitioned end to end.

## Window authority — the daemon is the brain

The daemon cannot spawn an OS window (no handle) — the Tauri shell adaptor does
that. But the daemon **owns window state and authority**: it mediates decisions
and holds the kill switch. Windows are self-registering lifecycle actors
(`ActorKind::Window`); a supervisor reaper closes the lifecycle of any whose pid
has died.

The window seam is a **hybrid**: the daemon mediates, the shell executes.

| op | tier | role |
|---|---|---|
| `SpaceService.WindowIntent(space_id)` | C0 | focus-vs-spawn decision (`find_live_window_for_space`); shell acts on it |
| `SpaceService.WindowList` | C0 | enumerate window actors — actor_id, space_id, pid, live |
| `SpaceService.CloseWindow(actor_id)` | **C3** | revoke the window's pid-bound token + close its lifecycle |

`CloseWindow` revokes by dropping every session matching the window's
`space_id`+`pid` — an instant authority cut: the webview is neutered even if its
OS frame lingers. It is **C3 (operator-only)** so a space shell (which holds C2)
cannot close another space's window.

## The two-token security model

A space webview authenticates with a **pid-bound C2 operator token**, minted
over the UDS host socket from the connection's kernel-attested `SO_PEERCRED` pid
(`register_space_window` → `space_window_session`). The token is two-factor: the
secret *and* being that exact process. A leaked token replayed from another
process fails `DaemonSession::pid_authorizes`.

- **C3 stays native**, core-held, never in JS. Space shells get up to **C2,
  never C3** — the gate (`services::gate::decide`) denies C3 unless `is_operator`,
  and a space-window session (`agent_id = "space:{id}"`) is structurally not the
  operator.
- The Connect transport for the webview routes through the native bridge over
  UDS — never raw `fetch` (TCP loopback has no peer-cred, so a bound token fails
  closed there).

## Config — global defaults, per-space overrides

Effective space config is **layered**: global daemon defaults overlaid per-key
by the space's own overrides. `SpaceService.Info` returns the active space's
identity (id/root/name) plus the merged `settings_json`.

- Per-space overrides are **event-sourced**: `space.setting.changed` →
  `space_settings` projection (keyed by `space_id`), written via the gated
  `space.setting.set` (C1). `Info` overlays them on the global `settings.state`
  snapshot at read time — space wins per key, global is untouched.
- Package-contributed config (a space depending on store apps) is the
  composition engine's job (separate, in progress) and composes *on top* of this.

## Where it lives

- Window actors, liveness, reaper: `ship-substrate` `projections::actor`,
  `db::actor_events`, the supervisor reaper.
- Space identity + per-connection emit: `ship-substrate::project::space`,
  `ship-platform::auth::scopes`, `apps/shipd/src/auth.rs`.
- Window token mint + pid-binding: `apps/shipd/src/auth/space_window.rs`,
  `peer_socket.rs`.
- The space service surface (Info/WindowIntent/WindowList/CloseWindow/SetSetting):
  `apps/shipd/src/rpc/space.rs` + `proto/ship/ops/v1/space.proto`.
- Per-space settings: `ship_config::projection::SpaceSettingsProjection`,
  `apps/shipd/src/rpc/space.rs`.
