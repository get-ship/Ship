---
group: Schema Reference
title: MCP Servers
section: reference
order: 5
---

# MCP Server Definitions — mcp.jsonc

**Location**: `.ship/mcp.jsonc`
**Schema**: `mcp.schema.json`

Shared MCP server definitions referenced by agent configs via `mcp.servers`. Agents select which servers to activate — unused servers are excluded from compiled output.

## Server entry fields

| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `name` | string | no | — | Human-readable server name |
| `command` | string | stdio: yes | — | Binary to run, e.g. `npx`, `ship`, `node` |
| `args` | string[] | no | — | Arguments passed to the server binary |
| `env` | object | no | — | Environment variables. Values support `${VAR}` expansion. |
| `url` | string (URI) | http/sse: yes | — | URL for remote transport servers |
| `server_type` | enum | no | `"stdio"` | `stdio`, `http`, or `sse` |
| `scope` | enum | no | `"project"` | `project` or `global` |
| `disabled` | boolean | no | `false` | Retained but excluded from compilation |
| `timeout_secs` | integer | no | — | Connection timeout in seconds |

## Provider-specific fields

| Field | Provider | Description |
|-------|----------|-------------|
| `codex_enabled_tools` | Codex | Tool names to enable |
| `codex_disabled_tools` | Codex | Tool names to disable |
| `gemini_trust` | Gemini | Whether this server is trusted |
| `gemini_include_tools` | Gemini | Tool names to include |
| `gemini_exclude_tools` | Gemini | Tool names to exclude |
| `gemini_timeout_ms` | Gemini | Per-server timeout in milliseconds |
| `cursor_env_file` | Cursor | Path to env file for stdio servers |

## Transport types

- **stdio** — runs a local process. Requires `command`.
- **http** — connects to a URL. Requires `url`.
- **sse** — connects to a URL with Server-Sent Events. Requires `url`.

## Example

```jsonc
{
  "$schema": "https://raw.githubusercontent.com/madvib/ship/main/schemas/mcp.schema.json",
  "mcp": {
    "servers": {
      "ship": {
        "name": "Ship",
        "command": "ship",
        "args": ["mcp", "serve"],
        "server_type": "stdio"
      },
      "github": {
        "name": "GitHub MCP",
        "command": "npx",
        "args": ["-y", "@modelcontextprotocol/server-github"],
        "server_type": "stdio",
        "env": { "GITHUB_TOKEN": "${GITHUB_TOKEN}" }
      },
      "remote-api": {
        "name": "Remote API",
        "url": "https://api.example.com/mcp",
        "server_type": "http"
      }
    }
  }
}
```
