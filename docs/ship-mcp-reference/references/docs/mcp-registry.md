---
group: HTTP API
order: 10
title: MCP Registry Proxy
description: Daemon endpoint that proxies the official MCP server registry with local caching and a curated fallback list.
---

# MCP Registry Proxy

`GET /api/mcp/servers`

Proxies the [Model Context Protocol server registry](https://registry.modelcontextprotocol.io) with a 24-hour in-memory cache. When the upstream registry is unreachable, returns a curated fallback list of vetted servers.

## Request

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `q` | string | — | Filter servers by name, id, description, or tags (max 200 chars) |
| `vetted` | bool | `true` | When `true`, returns only vetted servers. Pass `vetted=false` to include all |
| `limit` | int | 20 | Maximum results to return (1–100) |

## Response

```json
{
  "servers": [
    {
      "id": "github",
      "name": "GitHub",
      "description": "Search repos, manage PRs and issues, create commits.",
      "homepage": "https://github.com/modelcontextprotocol/servers/tree/main/src/github",
      "tags": ["popular", "dev"],
      "vendor": "Anthropic",
      "packageRegistry": "npm",
      "command": "npx",
      "args": ["-y", "@modelcontextprotocol/server-github"],
      "vetted": true,
      "imageUrl": "https://github.com/modelcontextprotocol.png"
    }
  ],
  "cached": true
}
```

`cached: true` means the response was served from the local cache without a live upstream fetch.

## Cache behavior

- **TTL**: 24 hours in-memory. The daemon caches the full unfiltered server list; query and vetted filters are applied in-memory on each request.
- **Stale-on-failure**: If the upstream fetch fails and a stale cache entry exists, the stale cache is returned (`cached: true`).
- **Cold start fallback**: If both the upstream and cache are unavailable, the curated fallback list is returned (`cached: false`).
- **No persistence**: The cache lives in daemon memory and resets on restart.

## Rate limiting

60 requests per minute (global token bucket). Exceeding the limit returns `429 Too Many Requests`.

## Curated fallback list

The fallback list is maintained in `apps/shipd/src/mcp_registry/fallback.rs`. It mirrors the `CURATED_FALLBACK` constant from the original `apps/web/src/lib/mcp-registry.ts`. To update it:

1. Edit `fallback.rs` — add, remove, or update `McpServer` structs.
2. Update `VETTED_IDS` in `apps/shipd/src/mcp_registry/mod.rs` if vetted status changes.
3. Run `cargo test -p shipd` to confirm the fallback tests still pass.

### Current curated servers

| ID | Name | Tags |
|----|------|------|
| github | GitHub | popular, dev |
| filesystem | Filesystem | popular, files |
| brave-search | Brave Search | search, web |
| memory | Memory | memory, context |
| slack | Slack | comms, popular |
| linear | Linear | pm, issues |
| playwright | Playwright | browser, testing |
| postgres | PostgreSQL | database, sql |
| puppeteer | Puppeteer | browser, scraping |
| sqlite | SQLite | database, local |

## Authentication

This endpoint is behind the standard daemon auth middleware. Callers must include a valid `Authorization: Bearer <token>` header, as with all `/api/*` daemon endpoints.
