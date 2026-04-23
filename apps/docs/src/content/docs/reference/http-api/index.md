---
title: "HTTP API"
description: "Shipd's local REST API — base URL, authentication, and error handling."
sidebar:
  label: "HTTP API"
  order: 0
---
`shipd` serves a local REST API on `http://127.0.0.1:7421` by default. Every request is authenticated with the bearer token in `~/.ship/daemon.token`; responses follow the OpenAPI contract committed to `apps/docs/openapi.json`.

- Per-operation pages (request/response schemas, examples, code samples) are generated from the OpenAPI spec and live under [`/reference/http-api/operations/`](/reference/http-api/operations/).
- Non-2xx responses all share a single envelope — see [Error Codes](/reference/http-api/errors/) for the canonical list and client-side handling guidance.

The spec is the source of truth: a drift check (`pnpm -C apps/docs run openapi:check-drift`) fails CI if the committed JSON diverges from what the running daemon advertises.
