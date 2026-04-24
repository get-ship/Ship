---
title: "Trigger Webhook Path Namespacing"
description: "ADR: user-authored webhook triggers bind under /webhooks/{trigger-id}/{path?} (local) and /webhooks/{tenant}/{trigger-id}/{path?} (cloud), separate from /api/v1/ and reserved from Ship internal routes."
sidebar:
  label: "Trigger Webhook Path Namespacing"
  order: 50
---
## Context

Ship triggers (`.ship/triggers/*.jsonc`, per the hooks→triggers rename tracked as audit Item 3) can subscribe to an external HTTP source. `schemas/trigger.schema.json` today declares the webhook source:

```jsonc
{
  "type": "webhook",
  "path": "/webhooks/...",
  "secret": "env:SOME_SECRET"
}
```

The schema requires paths to start with `/webhooks/`. The actual router in `apps/shipd/src/router.rs` today mounts webhooks at `/api/hooks/webhook/{*path}` — a leftover from the pre-rename naming. Neither location is reserved, neither namespaces by tenant, and neither separates user-authored paths from internal Ship API paths.

Three concrete problems arise:

1. **Collision between user-authored paths and Ship internal routes.** A trigger with `path: "/webhooks/workspaces"` would be fine today but collide the day Ship adds an internal `/webhooks/workspaces/*` endpoint. User-authored namespaces need a reservation, not an honor system.
2. **No tenant scoping.** In cloud, webhook paths must be disambiguated per tenant. A GitHub integration hitting `/webhooks/github` has to resolve to the *right* tenant's trigger; path alone cannot do that.
3. **Local ↔ cloud drift.** A trigger path that works against local `shipd` must work against cloud too. If the cloud requires a tenant prefix and local does not, the user's trigger definition is not portable.

Related audit items: Item 3 (hook/trigger rename and actions schema), Item 9 (API URL shape), Item 27 (cloud coherence). This ADR assumes the URL shape ADR and trigger rename are in flight.

## Decision

### 1. `/webhooks/*` is reserved for user-authored trigger webhooks

The `/webhooks/` top-level path is exempt from `/api/v1/` versioning (see the URL shape ADR) and reserved as the landing namespace for user-authored webhook triggers. Ship internal HTTP endpoints **never** live under `/webhooks/`. The router rejects any internal route registration under this prefix at build time.

### 2. Canonical shape

**Local:**

```
/webhooks/{trigger-id}
/webhooks/{trigger-id}/{subpath...}
```

**Cloud:**

```
/webhooks/{tenant}/{trigger-id}
/webhooks/{tenant}/{trigger-id}/{subpath...}
```

Where:

- `{tenant}` is the tenant identifier (same as used in `/api/v1/t/{tenant}/...` — see URL shape ADR).
- `{trigger-id}` is the trigger's `name` field from `.ship/triggers/{name}.jsonc` (or a stable per-trigger slug; see §4).
- `{subpath...}` is optional, captured verbatim and delivered to the trigger in the invocation context. Lets a single trigger handle multiple sub-paths (e.g., `/webhooks/github/push` and `/webhooks/github/pr` routed to one `github` trigger).

The `schemas/trigger.schema.json` `path` field is updated to accept exactly this shape. Existing `path: "/webhooks/foo"` entries continue to validate; freeform paths like `/webhooks/foo/bar/baz` are interpreted as trigger-id `foo` with subpath `bar/baz`.

### 3. Tenant segment is inserted by cloud routing, not authored

Trigger authors write:

```jsonc
{
  "name": "github-push",
  "trigger": {
    "source": {
      "type": "webhook",
      "path": "/webhooks/github-push"
    }
  }
}
```

The same trigger file works locally and in cloud. Cloud's edge router is responsible for mapping `https://hooks.ship.app/webhooks/{tenant}/github-push` to the tenant's trigger by inserting the `{tenant}` segment during dispatch. Authors never write the tenant segment; it is infrastructure, not contract.

Consequence: the external GitHub webhook URL is `https://hooks.ship.app/webhooks/{tenant}/github-push`; the URL the user copies into GitHub's webhook settings includes the tenant segment. The cloud UI surfaces the correct copy-paste URL.

### 4. Trigger ID is the trigger's `name` field, normalized

`trigger-id` in the path is the trigger's `name` field, normalized to kebab-case and lowercased, with `[a-z0-9-]` as the allowed character set. Duplicate normalized names across triggers in one scope fail schema validation at load time.

This keeps authoring simple (the `name` field serves three purposes: filename, identifier, URL segment) while guaranteeing URL safety.

### 5. Secret verification is trigger-scoped, not path-scoped

The `secret` field on a webhook source is checked against an `X-Ship-Webhook-Secret` header (or a source-specific signature scheme like GitHub's `X-Hub-Signature-256` — future extension). Verification happens *after* trigger resolution, not before. A missing or invalid secret returns 401; an unknown trigger returns 404. The distinction matters: an attacker probing paths sees 404 for nonexistent triggers and 401 for known-but-wrong-secret triggers, which is the expected ordering.

### 6. Method allowlist: POST by default; others opt-in

Webhook triggers accept `POST` by default (the overwhelming majority of webhook providers send POST). Triggers can opt in to additional methods via an optional `methods: ["POST", "PUT"]` field on the webhook source. `GET` is explicitly opt-in because it tempts confused-deputy and SSRF patterns if used carelessly. `OPTIONS` is handled by the router (CORS preflight), never routed to triggers.

### 7. Payload delivery and size limits

The request body is delivered to the trigger action context as bytes plus parsed JSON (when `Content-Type: application/json`). Headers are delivered as a map. Query string is delivered separately. The daemon enforces a body size limit (default 1 MiB, configurable per-trigger) and rejects oversize requests with 413.

## Consequences

### Positive

- **One namespace reservation prevents an entire class of future conflicts.** Internal Ship routes cannot grow into `/webhooks/`, and user-authored triggers cannot shadow internal routes.
- **Trigger definitions are portable.** Local-authored triggers run unchanged in cloud; the tenant segment is an infrastructure concern.
- **Cloud routing is a path-prefix match.** The cloud edge router routes `/webhooks/{tenant}/...` to the tenant's daemon with a simple prefix match. No header parsing, no tenant header required, no body inspection.
- **External providers stay happy.** The URL shape is stable and rule-of-thumb-memorable. Copy-paste into GitHub, Stripe, Linear, Slack works.
- **Collision is a load-time error, not a runtime surprise.** Duplicate trigger IDs fail schema validation when triggers are loaded, not when a webhook arrives.

### Negative

- **Cloud tenant segment is visible in the copied URL.** The URL a user pastes into GitHub includes `{tenant}`. If the tenant identifier changes (renaming), existing webhooks break. Mitigant: tenant IDs are stable UUIDs or slugs set at onboarding; renaming is an operator action with its own migration.
- **Normalizing `name` for URL safety restricts trigger names.** Names with characters outside `[a-z0-9-]` fail validation. Acceptable; schema errors are clear.
- **Subpath delivery adds a small API surface to trigger actions.** Actions need to access `{subpath}` from the invocation context. Documented; no contract cost.

## Alternatives Considered

### `/api/v1/triggers/{trigger-id}` — keep webhooks under the API prefix

Consistent with everything else, no special top-level namespace. Rejected for three reasons:

1. **`/api/v1/` implies "evolving with the Ship API version."** Webhook URLs are copy-pasted into third-party systems (GitHub, Stripe) by operators and are expected to be stable across Ship version bumps. Coupling them to the API version means a future `/api/v2/` migration breaks every registered external webhook.
2. **Webhooks are not part of the Ship public API contract.** External providers do not read the Ship OpenAPI spec; they call whatever URL the operator pasted. Including them in `/api/v1/` pollutes the OpenAPI surface with routes that are not meaningful to API consumers.
3. **Cloud routing simplicity.** `/webhooks/` as a dedicated top-level prefix lets the cloud edge route webhook traffic to a potentially different backend (higher body limits, different auth, async processing) than the `/api/v1/` surface. Separating them at the URL level makes that architecture choice possible without breaking URLs.

### Webhook-id-only, no tenant segment (cloud uses subdomain instead)

`https://{tenant}.hooks.ship.app/webhooks/{trigger-id}` — subdomain carries tenancy. Rejected for the same reasons the URL shape ADR rejected subdomain-based tenancy for the API: DNS/TLS cost, local parity break (no subdomains locally), and cookie semantics. The path-segment tenant keeps one model across `/api/v1/` and `/webhooks/`.

### Opaque per-webhook UUIDs — `/webhooks/{opaque-uuid}`

Give each trigger a random UUID and use it as the path. Authors copy the UUID into the third-party system.

Rejected. Loses human-readable debugging ("which trigger fired?" requires a UUID lookup). Loses the ability to rename a trigger in place — the UUID is permanent while the name is not. And loses the subpath-delivery pattern (§2) that lets one trigger handle multiple related events.

Kept as an *option*: triggers may specify `path: "/webhooks/_/{uuid}"` to opt into a UUID-based path for security-through-obscurity when the third-party system does not support secret headers. The `_` prefix reserves a sub-namespace for opaque webhook IDs. This is additive, not the default.

### Per-trigger port or socket

Bind each webhook trigger to a dedicated TCP port or Unix socket. Avoids path collisions entirely.

Rejected. Ports are a scarce resource, port allocation introduces firewall and cloud LB complexity, and third-party webhook URLs want a single stable host:port pair. The whole point of HTTP path routing is to multiplex one port; abandoning that on the webhook surface is a step backward.

### Route webhook paths through the tenant's primary API host

Mount webhooks at `/api/v1/t/{tenant}/triggers/{trigger-id}/invoke`. Share auth and routing with the API.

Rejected. Couples webhook URLs to API auth (third-party providers do not send Ship auth tokens) and to API versioning (noted above). Webhook traffic also has different characteristics — larger bodies, different retry semantics, different idempotency requirements — that merit a dedicated namespace even if they share a daemon binary.

## How to Measure Success

- `schemas/trigger.schema.json` `path` field validates against the documented shape (`^/webhooks/[a-z0-9-]+(/.*)?$` for local, cloud layer injects tenant).
- The router reserves `/webhooks/` exclusively for trigger dispatch. Adding an internal route under `/webhooks/*` fails at build time via a compile-time or startup check.
- A trigger authored locally (`path: "/webhooks/github-push"`) runs unchanged in cloud; the cloud edge prepends `{tenant}/` without requiring a file edit.
- Name collisions across triggers in one scope fail trigger loading with a clear error that names both offending files.
- Unknown trigger paths return 404; known triggers with missing or invalid secrets return 401. Body-size violations return 413.
- The docs surface a "Set up a webhook" reference page with the local and cloud URL shapes side by side.

## Open question (flagged for ratification)

The canonical cloud host for webhooks — `hooks.ship.app` vs `api.ship.app/webhooks/...` on the same origin — is a deployment decision outside the URL-shape contract. This ADR names `hooks.ship.app` as an illustrative example; the final hostname is set at cloud launch. Flagging because §3 references a specific host. If deployment lands on a single-origin model, §3 reads `https://api.ship.app/webhooks/{tenant}/...` with no substantive change to the contract. Recommendation: separate origin (`hooks.ship.app`) for the body-size and routing reasons in §2, but ratify at cloud-launch time.

## Cross-references

- Audit: `.artifacts/launch-audit/contract-freeze-menu.md` — Item 3 (hook/trigger rename), Item 27 (cloud coherence).
- Audit: `.artifacts/launch-audit/wire-and-storage.md` — Item 9 (API surface), context on the current `/api/hooks/webhook/{*path}` route that this ADR supersedes.
- ADR: `api-url-shape-and-versioning` — defines `/webhooks/*` as an exempt family; this ADR fills in the shape.
- ADR: `home-layout-and-scope-convention` — `.ship/triggers/` (project scope) and `library/triggers/` (user scope) are where trigger files live.
- ADR: `collections-as-data-layer` — triggers may invoke `collection.insert` / `collection.update` actions that land here.
- Schema: `schemas/trigger.schema.json` — the `path` field constraint updates to match §2.
