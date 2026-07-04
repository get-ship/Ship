import { defineAgent } from "ship";

// One profile per loop iteration — proves the collector rolls up N defineAgent
// calls to N AgentProfiles.
for (const id of ["reviewer", "tester", "builder"]) {
  defineAgent({
    agent: { id, name: id },
    capabilities: [{ domain: "workspace" }],
  });
}
