import { defineAgent, globals } from "ship";

// Injected default lands on every agent.
defineAgent({
  ...globals.agentDefaults,
  agent: { id: "solo", name: "solo" },
});

// Same injected default, but a per-agent field overrides it — local wins.
defineAgent({
  ...globals.agentDefaults,
  agent: { id: "override", name: "override" },
  target_agents: ["local-only"],
});
