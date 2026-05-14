---
group: MCP Tools
order: 1
title: MCP Tools
description: Ship's MCP server exposes runtime operations to AI agents via the Model Context Protocol.
---

<ship-alert tone="warning">
<span slot="title">Reference under revision</span>
The full tool catalog and per-tool reference are being reworked. The shape below is current; detailed pages are temporarily internal-only.
</ship-alert>

> Note: the MCP tool surface is under active revision; tool/resource lists below may be out of date.

Ship runs an MCP server (`apps/mcp/`) that exposes its runtime — workspaces, sessions, agents, jobs, and project state — to AI agents over the Model Context Protocol. Any MCP-aware tool can connect.

## Server

`ShipServer` is the MCP surface. Started via `ship mcp serve` (stdio) or `ship mcp serve --http --port <N>` (HTTP).

## How agents see it

The standard configuration is generated for you by `ship use`. It writes an `.mcp.json` file your CLI agent picks up automatically:

```json
{
  "mcpServers": {
    "ship": { "command": "ship", "args": ["mcp", "serve"] }
  }
}
```

When the agent starts, it sees only the tools the active agent profile allows via `active_tools`.
