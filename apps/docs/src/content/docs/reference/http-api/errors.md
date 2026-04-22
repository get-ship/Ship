---
title: "Error Codes"
description: "Ship's canonical ErrorResponse codes — meaning, HTTP status, and remediation."
sidebar:
  label: "Error Codes"
  order: 1
---
Every non-2xx response from the Ship daemon conforms to the
[`ErrorResponse`](/reference/http-api/components/schemas/errorresponse/)
schema:

```json
{
  "error": "not_found",
  "message": "spec not found: introductions",
  "details": null
}
```

- `error` — machine-readable code. Stable; safe to branch on.
- `message` — human-readable description. Safe to surface in UIs and logs.
- `details` — optional structured context. Shape varies by endpoint.

The HTTP status code carries the coarse category (`4xx` client, `5xx`
server); the `error` field narrows it to the specific cause.

## Error codes

| Code                    | HTTP | Meaning                                         | Common causes                                                   | Remediation                                                                      |
| :---------------------- | :--: | :---------------------------------------------- | :-------------------------------------------------------------- | :------------------------------------------------------------------------------- |
| `bad_request`           | 400  | Malformed request or invalid parameters.        | JSON parse failure, missing required field, wrong value type.   | Re-read the endpoint schema; validate payloads client-side before sending.       |
| `validation_failed`     | 400  | Payload parsed but failed business validation.  | Empty required string, negative integer where positive expected.| Check the `details` object for per-field errors and correct the payload.         |
| `unauthorized`          | 401  | Bearer token missing or invalid.                | No `Authorization` header; expired session; revoked token.      | Reload `~/.ship/daemon.token` or re-pair via `POST /auth/pair/init`.             |
| `forbidden`             | 403  | Authenticated, but caller lacks required scope. | Session role maps to scopes that don't include the one needed.  | Issue a new session with a role that grants the missing scope (e.g. `lead`).     |
| `not_found`             | 404  | Target resource does not exist.                 | Wrong id, deleted entity, spec-by-slug missing.                 | Verify the id/slug; list the parent collection to confirm the resource exists.   |
| `conflict`              | 409  | State conflicts with the request.               | Creating an entity whose id is already taken; stale write.      | Fetch current state and retry with the correct version / different id.          |
| `already_started`       | 409  | A one-shot operation was re-issued.             | Starting a workspace that's already running; re-spawning actor. | Treat the original operation as authoritative; skip the retry.                   |
| `agent_not_registered`  | 409  | Mesh/actor operation targets an unknown agent.  | Sending a mesh message to an agent that hasn't registered.      | Call `POST /api/mesh/register` for the agent first, then retry.                  |
| `rate_limited`          | 429  | Too many requests in the current window.        | MCP registry proxy; webhook ingress burst.                      | Back off (the response includes a `Retry-After` header when applicable).         |
| `internal`              | 500  | Unhandled server-side failure.                  | DB error, panic caught by an axum handler, unexpected I/O.      | Capture `details` and the daemon logs; file a bug against the emitting handler. |

### Per-endpoint extensions

Some endpoints emit narrower codes documented on their individual pages:

- `POST /auth/pair/confirm` returns `expired` / `unknown` in the `details`
  block when the pairing code is invalid.
- `POST /api/hooks/webhook/{path}` returns a minimal `{ ok: true }` body on
  success; failures follow the `ErrorResponse` shape with `bad_request` or
  `not_found`.

## Client guidance

```js
const res = await fetch(url, { /* … */ });
if (!res.ok) {
  const err = await res.json();
  switch (err.error) {
    case "unauthorized":
      // refresh token and retry
      break;
    case "not_found":
      // treat as empty result
      break;
    case "rate_limited": {
      const retry = Number(res.headers.get("Retry-After") ?? "1");
      await new Promise((r) => setTimeout(r, retry * 1000));
      // retry
      break;
    }
    default:
      throw new Error(`${err.error}: ${err.message}`);
  }
}
```

Never parse `message` for control flow — branch on `error`. Messages are
written for humans and may change between daemon versions.
