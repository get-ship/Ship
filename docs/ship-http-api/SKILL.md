---
name: ship-http-api
stable-id: ship-http-api
description: Reference docs for Ship's legacy local REST API (shipd) — base URL, auth, error codes, OpenAPI operation pages. The canonical surface is now the ConnectRPC op surface; REST is retiring.
tags: [ship, http-api, reference, legacy]
authors: [ship]
---

# Ship HTTP API

> **The canonical daemon surface is now the [ConnectRPC op surface](/architecture/op-surface/)** — every operation is a method in a per-domain `ship.ops.v1` service, authored from proto. This REST/OpenAPI surface is **legacy**: it still serves the GUI and is described here, but it is being retired once the GUI flip lands (#78). New clients use the generated Connect clients (`@ship/sdk`) or `ship ops call`, not REST.

`shipd` still exposes this local REST API (the GUI rides it until the Connect flip, plus protocol-mandated edges). The surface is described by a versioned OpenAPI document and rendered into per-operation pages at build time.

- **[Overview](/http-api/)** — base URL, authentication, error handling.
- **[Error Codes](/http-api/errors/)** — canonical `ErrorResponse` codes, HTTP statuses, and remediation.
- **Operations** — auto-generated from `apps/docs/openapi.json` under `/api-reference/operations/`.
