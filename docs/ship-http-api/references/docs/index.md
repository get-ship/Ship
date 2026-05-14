---
group: HTTP API
order: 0
title: HTTP API
description: Shipd's local REST API — base URL, authentication, and error handling.
audience: internal
---

`shipd` serves a local REST API on `http://127.0.0.1:9315` by default. Every request is authenticated with the bearer token in `~/.ship/daemon.token`; responses follow the OpenAPI contract committed to `apps/docs/openapi.json`.

- Per-operation pages (request/response schemas, examples, code samples) are generated from the OpenAPI spec and live under [`/http-api/operations/`](/http-api/operations/).
- Non-2xx responses all share a single envelope — see [Error Codes](/http-api/errors/) for the canonical list and client-side handling guidance.

The spec is the source of truth: a drift check (`cd apps/docs && bun run openapi:check-drift`) fails CI if the committed JSON diverges from what the running daemon advertises.
