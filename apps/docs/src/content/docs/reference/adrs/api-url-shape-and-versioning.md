---
title: "API URL Shape and Versioning"
description: "ADR: all HTTP API routes live under /api/v1/; tenancy is a path segment (/api/v1/t/{tenant}/...); auth, assets, and webhooks are exempt from the /api prefix."
sidebar:
  label: "API URL Shape and Versioning"
  order: 30
---
## Context

Audit Item 9 (`.artifacts/launch-audit/wire-and-storage.md`) is the most irreversible surface in Ship's pre-launch contract freeze. The HTTP API today exposes 137+ routes across `apps/shipd/src/router.rs` with no version prefix, no tenancy addressing, no published OpenAPI, and no CI guard against drift.

Route families present today:

- `/auth/*` — pairing, sessions, attestation.
- `/api/mesh/*`, `/api/actors/*`, `/api/events/*`, `/api/workspaces`, `/api/sessions`, `/api/agents`, `/api/jobs/*` — core runtime surface.
- `/api/views`, `/api/v/:name`, `/api/view/:name/*` — view rendering, including SSE streams.
- `/api/collections/*`, `/api/artifacts/*`, `/api/hooks/*`, `/api/feedback`, `/api/annotations` — data layer.
- `/api/assets/*` — static asset serving.
- `/api/hooks/webhook/{*path}` — user-authored webhook triggers (see the trigger namespacing ADR for why this path is wrong).
- `/api/ship/schema` — SDK surface schema (see audit Item 13).

Three facts force this decision now:

1. **Versioning is retroactive or it is impossible.** Adding `/api/v1/` *after* external callers (cloud SDKs, CLI, MCP, published views, webhooks) pin today's shape means carrying both forever.
2. **Cloud needs tenancy.** A single `shipd` binary today addresses one local user. The cloud deployment addresses N tenants over one router. The tenancy identifier has to sit *somewhere* in the URL shape (path, subdomain, or header) and that choice constrains every SDK and every cloud router rule downstream.
3. **Local ↔ cloud parity matters.** A URL that works against `http://127.0.0.1:PORT` must work (modulo host) against `https://{something}.ship.app`. If the tenancy segment only exists in cloud, every SDK either branches on environment or the local daemon pretends to serve a single tenant named `local`.

This ADR locks in the URL shape for v1. Related: `collections-as-data-layer` (every collection CRUD route needs a home under this prefix), `home-layout-and-scope-convention` (the `machine/daemon/` handshake exposes the port local callers hit).

## Decision

### 1. `/api/v1/` is the prefix for every versioned route

Every HTTP API route that belongs to the **Ship public contract** lives under `/api/v1/`. Example shapes:

```
/api/v1/workspaces
/api/v1/workspaces/{id}
/api/v1/workspaces/{id}/sessions
/api/v1/jobs/{id}
/api/v1/jobs/{id}/log          (SSE)
/api/v1/collections/{id}/items
/api/v1/artifacts/{id}
/api/v1/views
/api/v1/view/{name}
/api/v1/view/{name}/sse         (Datastar signal stream)
/api/v1/ship/schema
/api/v1/openapi.json
```

The prefix is unconditional. There is no "prefix-less for convenience" escape hatch. Unversioned routes either belong to an exempt family (below) or they are bugs.

### 2. Exempt families: `/auth/*`, `/assets/*`, `/webhooks/*`, `/health`

Four route families sit outside `/api/v1/`:

| Prefix | Why exempt |
|---|---|
| `/auth/*` | Version-independent: pairing and attestation are the bootstrap that *discovers* which API versions a daemon supports. Must not require knowledge of the API version to establish an auth session. |
| `/assets/*` | Static content. Content-addressed by filename hash; versioning the URL adds nothing. |
| `/webhooks/{tenant?}/{trigger-id}/{path?}` | User-authored external webhook landing paths. Namespace belongs to *user triggers*, not the Ship API contract. See the trigger webhook namespacing ADR. |
| `/health` | Liveness probe consumed by orchestrators (systemd, k8s, Cloudflare health checks). Stable URL required; contents may evolve. |

No other exemptions. If a new route does not fit one of these four families, it lives under `/api/v1/`.

### 3. Tenancy is a path segment: `/api/v1/t/{tenant}/...`

Cloud deployments address a tenant via a `t/{tenant-id}` segment immediately after the version:

```
/api/v1/t/{tenant}/workspaces
/api/v1/t/{tenant}/jobs/{id}
/api/v1/t/{tenant}/collections/{id}/items
```

Local `shipd` **does not** emit the `t/{tenant}/` segment. Local routes are:

```
/api/v1/workspaces
/api/v1/jobs/{id}
```

The absence of the tenant segment in local routes is the marker that this daemon is single-tenant. Cloud routers accept both shapes: `t/{tenant}/...` routes to the tenant's workspace; bare routes 404 (cloud has no "current user" inference — tenant must be explicit).

SDKs compose the tenant segment from a configured tenant ID. A `local` SDK configuration omits it; a `cloud` SDK configuration prepends `t/{tenant}`. The operation names and argument shapes are identical in both modes.

### 4. OpenAPI spec is the single source of truth

`apps/docs/openapi.json` is the published contract. It is regenerated from the router at build time (`apps/docs/scripts/generate-openapi.ts`) and committed. A CI guard (`apps/docs/scripts/check-openapi-drift.ts`) fails when the router and the spec disagree.

Every route under `/api/v1/` appears in the spec. Every route in the spec maps to a `utoipa::path!` declaration in the router. Drift is a CI failure, not a docs task.

### 5. SSE endpoints use the same prefix

View stream endpoints (`/api/v1/view/{name}/sse`) and job log endpoints (`/api/v1/jobs/{id}/log`) use the same `/api/v1/` prefix. Content negotiation is by `Accept: text/event-stream` at the route level, not by URL prefix. No separate `/sse/` or `/stream/` top-level namespace.

## Consequences

### Positive

- **v2 is possible.** We can introduce `/api/v2/` side-by-side when a breaking change is required, deprecate v1 on a published schedule, and remove it without carrying forever.
- **Local ↔ cloud parity.** One SDK, two configurations. The tenancy segment is a concatenation concern, not a branching concern.
- **OpenAPI is real.** Every external consumer — CLI, MCP, cloud SDK, third-party integrations — reads from one published spec. Code generators work out of the box.
- **CI guard prevents drift.** The router and the spec cannot silently diverge; the guard job fails PRs that add routes without registering them.
- **Cloud routing is trivial.** The cloud edge router matches `/api/v1/t/{tenant}/...` and routes to the tenant's `shipd` backend by tenant ID. No header parsing, no subdomain DNS, no per-region cert dance.

### Negative

- **Every existing caller migrates.** CLI, MCP, views, internal daemons. Breaking. Mitigant: this lands *before* v1 launch, when there are no external users to break.
- **Longer URLs.** `/api/v1/workspaces` instead of `/api/workspaces`. A fixed five-character cost for the contract.
- **The `t/` segment conflicts with legitimately-named resources called "t".** Mitigated by the fact that `t/` is the *first* path segment after `/api/v1/` and is reserved by convention; no Ship resource will ever be named `t`.
- **Local-cloud SDK symmetry requires discipline.** If a route exists on local but not cloud (or vice-versa), the OpenAPI spec has to encode that. We accept this as the price of one unified shape.

## Alternatives Considered

### `/v1/api/...` — version before "api"

Used by some APIs (Stripe historically used `/v1/`). Rejected: the `/api` segment is the meaningful distinguisher from `/auth`, `/assets`, `/webhooks`, `/health`. Putting `/v1` first orphans the version from the thing it versions, and conflicts with the pattern of having `/v1/ship/schema` mean "version-1-of-ship" rather than "version-1-of-api". `/api/v1/` keeps the scope and version visually coupled.

### Unversioned `/api/...` with an `API-Version:` header

Header-based versioning (GitHub's approach for a period). Rejected on two grounds:

1. **Caching and routing layers are URL-centric.** Cloud edge (Cloudflare, Fastly, Varnish) caches and routes by URL. Header-based versioning defeats the cache and forces every layer to vary on a custom header.
2. **Debuggability.** A URL containing `v1` tells an operator which API version a request hit. A header requires capturing request metadata to answer the same question. For an API that SREs and users will debug together, the URL wins.

### Tenancy via subdomain: `{tenant}.ship.app/api/v1/...`

Clean visually, standard in multi-tenant SaaS (Slack, Stripe Connect). Rejected for v1:

1. **DNS and TLS cost.** Every tenant gets an SSL cert. Cloudflare-for-SaaS or equivalent adds operational cost. For a tool that also runs a free local daemon, forcing subdomain routing in cloud adds a split local-vs-cloud auth story (local has no subdomain).
2. **Local parity breaks.** `http://127.0.0.1:PORT/api/v1/...` cannot carry a subdomain. SDKs would branch on environment.
3. **Cookies.** Subdomain-scoped cookies have sharing semantics that a path-scoped tenant segment avoids. One less footgun.

We can add subdomain routing later as a convenience layer that rewrites to `/api/v1/t/{tenant}/...` internally without changing the canonical URL shape.

### Tenancy via `X-Tenant-Id` header

Rejected for the same reasons as header-based versioning: defeats URL-based caching and routing, hurts debuggability. Additionally, it makes the tenant identifier easy to forget, and forgetting it in a cloud context means "act as no tenant" (500) or "act as the authenticated user's default tenant" (correct-looking but ambiguous). A missing path segment is an obvious 404; a missing header is an ambiguous error.

### Unversioned API forever (`/api/...`)

Considered. Rejected: Ship's public contract is young but not soft. Views, MCP clients, and the CLI will pin these URLs at launch. Without a version prefix, every breaking change requires a parallel route for the remainder of the project's life. The cost of the prefix is five characters; the cost of omitting it is permanent.

### Local hardcodes `t/local/`

Considered as a way to make local and cloud URL shapes identical. Rejected: local `shipd` has no tenancy concept — there is exactly one user. A synthetic `t/local/` segment adds noise to every local URL, forces the daemon to validate a tenant ID that can only ever be one value, and teaches developers to ignore the tenant segment ("it's just `local`") — which is exactly the wrong habit to build before they move to cloud. Omitting the segment locally preserves the signal that tenancy only matters in cloud.

## How to Measure Success

- Every route in `apps/shipd/src/router.rs` either lives under `/api/v1/` or belongs to one of the four exempt families. A lint/CI check enumerates exceptions; exceptions require an explicit allow-list entry.
- `apps/docs/openapi.json` exists, is committed, matches the router exactly. `pnpm -F @ship/docs openapi:check-drift` exits 0.
- Cloud deployment serves the same routes with a `t/{tenant}/` segment inserted, generated by the edge router. No `shipd`-side code change is required to support tenancy — it is an edge concern (for v1).
- The CLI and MCP callers resolve URLs through a single helper that knows the `/api/v1/` prefix. `grep` for `/api/` outside that helper returns zero hits in caller code.
- Introducing v2 in the future requires adding `/api/v2/` routes; it does not require renaming v1 routes or rewriting callers still on v1.

## Cross-references

- Audit: `.artifacts/launch-audit/wire-and-storage.md` — Item 9.
- Audit: `.artifacts/launch-audit/contract-freeze-menu.md` — Item 9 (change-before-freeze), Item 27 (cloud coherence).
- ADR: `collections-as-data-layer` — every collection CRUD endpoint is hosted under this prefix.
- ADR: `home-layout-and-scope-convention` — `machine/daemon/port` + `machine/daemon/token` is how local callers discover the API host this prefix attaches to.
- Related ADR: `trigger-webhook-path-namespacing` — defines the `/webhooks/*` exempt family in detail.
- Related ADR: `artifact-storage-key-opacity` — `/api/v1/artifacts/{id}` serves storage keys whose format is not part of the URL contract.
