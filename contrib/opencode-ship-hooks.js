// Ship lifecycle hooks plugin for OpenCode.
//
// Reads daemon connection info from ~/.ship/daemon.{port,token} and
// forwards tool execution events to the shipd hook endpoints.
//
// Usage: reference this file in opencode.json:
//   { "hooks": { "file": "contrib/opencode-ship-hooks.js" } }

import { readFileSync } from "node:fs";
import { join } from "node:path";
import { homedir } from "node:os";

function readDaemonInfo() {
  const shipDir = join(homedir(), ".ship");
  try {
    const port = readFileSync(join(shipDir, "daemon.port"), "utf8").trim();
    const token = readFileSync(join(shipDir, "daemon.token"), "utf8").trim();
    return { port: Number(port), token };
  } catch {
    return null;
  }
}

async function postHook(base, token, endpoint, body) {
  try {
    const resp = await fetch(`${base}/api/hooks/${endpoint}`, {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        Authorization: `Bearer ${token}`,
      },
      body: JSON.stringify(body),
      signal: AbortSignal.timeout(5000),
    });
    if (resp.ok) {
      return await resp.json();
    }
  } catch {
    // Daemon unavailable — silently ignore.
  }
  return { action: "allow" };
}

export const ShipHooks = async ({ directory }) => {
  const daemon = readDaemonInfo();
  if (!daemon) {
    // No daemon running — return no-op hooks.
    return {};
  }

  const base = `http://127.0.0.1:${daemon.port}`;
  const token = daemon.token;
  const actorId = process.env.SHIP_ACTOR_ID || "opencode";
  const workspace = process.env.SHIP_WORKSPACE_ID || "";
  const sessionId = process.env.SHIP_SESSION_ID || "";

  return {
    "tool.execute.before": async (input) => {
      const body = {
        tool_name: input?.tool || input?.name || "unknown",
        tool_input: input?.input || input?.args || {},
        actor_id: actorId,
        workspace,
        session_id: sessionId,
      };
      const result = await postHook(base, token, "before-tool", body);
      if (result.action === "block") {
        return { blocked: true, reason: result.reason || "blocked by ship" };
      }
      return undefined;
    },

    "tool.execute.after": async (input, output) => {
      const body = {
        tool_name: input?.tool || input?.name || "unknown",
        tool_input: input?.input || input?.args || {},
        tool_response: output || {},
        actor_id: actorId,
        workspace,
        session_id: sessionId,
      };
      await postHook(base, token, "after-tool", body);
    },
  };
};
