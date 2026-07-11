// Slice 2c: the helmd Compile integration test's on-disk agent file
// (the manifest is a thin pointer; per-noun source dirs hold the
// real content). The `defineAgent` body is byte-same to the W5
// inline form the fixture previously carried.
import { defineAgent } from "ship";

defineAgent({
  agent: {
    id: "test-agent",
    name: "Test Agent",
    version: "0.1.0",
    description: "An agent used by the helmd Compile integration test.",
  },
  skills: {
    refs: ["example-skill"],
  },
  mcp: {
    servers: ["test-server"],
  },
  rules: {
    inline: "Test rule body for helmd Compile integration test.",
  },
});
