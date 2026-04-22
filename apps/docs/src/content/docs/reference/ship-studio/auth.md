---
title: "ship-studio"
---
Studio runs in your browser. The Ship daemon runs on your computer. Authentication binds
the two together so only you can control your daemon from Studio.

Studio tries three auth methods in this order:

1. **URL fragment token** — included when you run `ship studio` in a terminal
2. **Server attestation** — zero-click if you're signed in at getship.dev
3. **Pairing code** — one-time setup, works offline and self-hosted

## Zero-click: sign in at getship.dev

If you're signed in at getship.dev and have run `ship login` on this machine, Studio
connects automatically — no code to copy, nothing to type.

**How it works:**
- Studio asks getship.dev for a signed attestation token (JWT)
- Studio forwards the token to your local daemon
- Your daemon verifies the signature against getship.dev's public key
- The daemon checks: is the signed user the same user who ran `ship login` on this
  machine? If yes, it issues a session token.

The daemon caches getship.dev's public key for an hour. Verification works offline once
cached — you only need internet when the JWT expires (every hour) to fetch a fresh one.

If the daemon rejects the attestation (different user, or you never ran `ship login`
here), Studio falls back to the pairing code flow.

## First time: pair this machine

When you open Studio and the daemon is running, Studio shows a **pairing code**:

```
  4F8-2K9

  ship auth pair 4F8-2K9
```

Copy the command, paste it into your terminal, and hit enter. Studio connects automatically.

This happens once per browser. The session lasts 30 days. If you sign in from a different
browser (or the same browser after clearing storage), you'll see a new code.

## Why the code?

Studio is a web app. It can reach your local daemon through the browser, but the daemon
has no way to tell "this is the real human using the computer" vs. "this is a malicious
script running on localhost."

The pairing code solves this. The daemon shows the code to Studio and waits. The user
proves they control the computer by running `ship auth pair <code>` in a terminal —
something only the real user has access to. Once confirmed, the daemon issues a session
token that Studio uses for the next 30 days.

## Seeing and revoking sessions

List all active paired sessions:

```
$ ship auth status
TOKEN               AGENT                 KIND        EXPIRES
ship_sess_abc123de  studio                persisted   in 29d 12h
ship_sess_def456fa  studio-dev            ephemeral   never
```

Revoke a specific session (kicks that browser out):

```
$ ship auth revoke --token ship_sess_abc
Revoked 1 session(s).
```

Revoke all sessions (kicks every paired browser):

```
$ ship auth revoke --all
Revoked 2 session(s).
```

## What if the code expires?

Pairing codes expire in **5 minutes**. If you take longer, Studio gets a fresh code
automatically. Click "new code" in the Studio pairing screen to force a restart sooner.

## What if the session expires?

Sessions last 30 days. When yours expires, Studio shows the pairing screen again — your
cached session is dropped and you re-pair once.

Studio also re-pairs automatically if the daemon ever returns a 401 (session revoked,
daemon reset, etc.). No page reload needed.

## Development mode: skip the pairing

If you're iterating on Ship itself and don't want to type pairing codes all day, start
the daemon in dev mode:

```
ship daemon start --dev
```

In dev mode:

- Studio auto-pairs on the first request — no code, no CLI confirmation
- The daemon forces localhost-only binding (no LAN access)
- A loud warning banner prints on startup
- Every relaxed auth decision is logged with a `WARN`

Dev mode is only enabled by the `--dev` flag on the daemon process. An agent process
can't enable it — only the human starting the daemon can.

**Never run production with `--dev`.** It's deliberately unsafe for convenience during
development.

## `ship studio` — pre-paired launch

If you want to launch Studio from a terminal with an already-paired session:

```
ship studio
```

This starts Studio in your browser with the daemon token included in the URL fragment.
No pairing needed — you've already proven you control the machine by being in a terminal.
This is equivalent to a permanent pairing and bypasses the 30-day expiry.

## Self-hosted / air-gapped

The pairing flow has **no dependency on getship.dev or any internet service**. Everything
happens between your browser and your local daemon. You can use Studio completely offline
— self-hosted, on an air-gapped machine, on a plane.

The only thing the pairing flow requires is:
- A running daemon (`ship daemon start`)
- A terminal on the same machine (to run `ship auth pair`)
- A browser that can reach the daemon URL (default `http://localhost:9315`)
