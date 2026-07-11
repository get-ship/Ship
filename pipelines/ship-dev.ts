// pipelines/ship-dev.ts — per-job source dir for the `ship-dev` pipeline.
// Migrated from `ship.config.ts`'s inline `definePipeline` call (slice 2c,
// job-spec-manifest-extraction). Byte-same payload; inline prompts stay
// verbatim.
import { definePipeline } from "ship";

definePipeline({
  name: "ship-dev",
  steps: [
    {
      id: "implement",
      step_specs: [{ id: "implement", agent: "rust-lane", harness: "claude" }],
    },
    {
      id: "verify",
      after: ["implement"],
      step_specs: [{ id: "verify", prompt: "cargo check --workspace 2>&1 | tail -5\ncargo test --workspace --no-fail-fast 2>&1 | tail -10\n" }],
    },
    {
      id: "review",
      after: ["verify"],
      step_specs: [{ id: "review", agent: "gatekeeper" }],
      gate: {
        require_changed: ["**/*.rs"],
        forbid_changed: [".artifacts/**"],
      },
    },
  ],
  auto_merge: true,
});
