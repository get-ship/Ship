---
audience: internal
status: design
---

# Per-user local TLS

**Status:** design captured, not implemented. Not blocking launch.
**Triggers for re-prioritization:** multi-user-per-machine scenarios become
common; cross-device sync needs a stable internet-routable identity;
desktop-app context where browser TLS chrome (the lock icon) matters even
for local connections.

## Problem

Ship's daemon serves HTTP on `localhost:<random-port>`. Studio, Shell, and
external integrations talk to it from a browser. Today this works because:

- `localhost` is a [Secure Context](https://w3c.github.io/webappsec-secure-contexts/)
  per W3C — `crypto.subtle`, `ServiceWorker`, sensors, etc. all work
  without HTTPS
- Bearer-in-Authorization-header sidesteps CSRF (browsers don't auto-include
  it cross-origin)
- CORS allowlist + Origin validation cover the cookie-authed cases

Problems this leaves unsolved:

1. **No browser TLS chrome.** The user never sees a lock icon. For a
   product asking them to install agentic apps, this is a real trust signal
   gap.
2. **Cookie scope leaks across localhost services.** Cookies set on
   `localhost` are visible to any other process bound to localhost. A second
   tool the user runs (or a malicious download) could read Ship cookies.
3. **Multi-user-per-machine doesn't work.** Two Ship users on the same
   machine share the same `localhost` and same cookie jar. No per-user
   isolation.
4. **No stable internet-routable identity.** Cross-device sync (Ship Cloud)
   needs daemons to identify each other; `localhost:<random-port>` isn't
   routable, and `127.0.0.1` isn't unique.

## Proposed shape

Assign each Ship account a unique subdomain under `getship.dev`:

```
<userid-hash>.local.getship.dev
```

Wildcard A record `*.local.getship.dev` → `127.0.0.1` at Cloudflare. One
DNS record, no per-user setup, no `/etc/hosts` edit. Costs nothing.

TLS cert issued by **Let's Encrypt via DNS-01** challenge. HTTP-01 won't
work because the hostname resolves to localhost (the LE validator can't
reach it). DNS-01 is solved by automating a TXT record at Cloudflare for
the challenge — Ship's gateway worker does this when a user's daemon
requests a cert.

Cert distributed to the daemon on first login + refresh on rotation
(~60 days for LE). Daemon serves HTTPS on a high port at
`<userid-hash>.local.getship.dev:<port>`.

The browser sees a real origin with valid TLS. Cookies set on that origin
don't bleed to other localhost services. Multi-user works because each
user gets a unique hostname. Cross-device sync can use the same naming
convention to address daemons.

## Why account-required

This is the structural reason the earlier discussion landed on "free tier
may need an account for Shell↔daemon secure connection." The per-user
subdomain requires server-side identity assignment, which requires an
account.

Counter-argument: free tier still works with bearer-over-HTTP-on-localhost.
The standard hardening (Authorization header, CORS, Origin, CSP) carries
the security floor without HTTPS. Per-user TLS is an upgrade for paid
tiers or for the multi-tenant case — not a baseline requirement.

So: free tier stays on the current shape; per-user TLS is an opt-in
upgrade when the user creates an account. Documented as such.

## Component sketch

```
┌─────────────────────────────────────────────────────────────────────┐
│ Cloudflare                                                          │
│                                                                     │
│  *.local.getship.dev → 127.0.0.1   (wildcard A record, one config)  │
│  ACME DNS-01 challenge handler     (api.getship.dev worker)         │
└─────────────────────────────────────────────────────────────────────┘
                          │
                          │ DNS-01 challenge / cert delivery
                          ▼
┌─────────────────────────────────────────────────────────────────────┐
│ User's machine                                                      │
│                                                                     │
│  shipd → HTTPS on <userid>.local.getship.dev:<port>                 │
│       ↑                                                             │
│       │ cert auto-installed from gateway on first login,            │
│       │ rotated ~60 days                                            │
│                                                                     │
│  Browser → trusts cert because LE chain is in system roots          │
│         → real lock icon, real Origin, real cookie scope            │
└─────────────────────────────────────────────────────────────────────┘
```

## Implementation cost (honest estimate)

| Piece | Effort |
|---|---|
| Cloudflare wildcard DNS A record | ~5 min, one-time |
| ACME DNS-01 challenge handler on gateway worker | 1-2 days (acme client, Cloudflare API integration) |
| Cert storage + refresh logic on daemon | 1-2 days (cert reload without downtime) |
| Daemon HTTPS server (axum-rustls) | <1 day |
| Studio/Shell URL rewrites for the new origin | 1 day |
| Testing + edge cases (cert expiry, network outages, account deletion) | 1-2 days |

Total: ~1 week of focused work. Not blocking launch; useful when multi-user
or cross-device becomes pressing.

## What this does not solve

- **Browser-to-daemon trust for unauthenticated users.** A user without an
  account still gets the bearer-over-HTTP path. Per-user TLS is the
  upgrade, not the baseline.
- **Process-to-process auth on the same machine.** Different problem
  (Unix socket + SO_PEERCRED is the answer there).
- **Sandbox escape from a compromised agent.** Different problem
  (OS sandbox / VM-per-agent — see SECURITY.md Layer 3).

## Decision log

- **2026-05:** Captured during the daemon-account auth refactor when the
  Shell↔daemon secure-connection question surfaced. Concluded not needed
  for launch (standard hardening suffices) but worth filing because the
  per-user subdomain idea has come up multiple times and the design wasn't
  written down anywhere.
