import { defineAgent } from "ship";
// `import type` — erased entirely by the strip (type-only import).
import type { AgentProfile } from "ship";

// Real type annotation + `satisfies` — both TS-only, both stripped to plain JS.
const base: Partial<AgentProfile> = {
  capabilities: [{ domain: "workspace" }],
} satisfies Partial<AgentProfile>;

// Typed params + return annotation — stripped, body preserved.
function make(id: string, name: string): void {
  defineAgent({
    ...base,
    agent: { id, name },
  });
}

// `as const` assertion — stripped; the loop runs as ordinary JS.
for (const id of ["alpha", "beta"] as const) {
  make(id, id);
}
