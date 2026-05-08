---
name: ship-http-api
stable-id: ship-http-api
description: Reference docs for Ship's local HTTP API (shipd) — base URL, authentication, canonical error codes, and auto-generated operation pages from the OpenAPI spec.
tags: [ship, http-api, reference]
authors: [ship]
---

# Ship HTTP API

`shipd` exposes a local REST API used by the CLI, MCP server, Studio, and third-party clients. The surface is described by a versioned OpenAPI document and rendered into per-operation pages at build time.

- **[Overview](/http-api/)** — base URL, authentication, error handling.
- **[Error Codes](/http-api/errors/)** — canonical `ErrorResponse` codes, HTTP statuses, and remediation.
- **Operations** — auto-generated from `apps/docs/openapi.json` under `/http-api/operations/`.
