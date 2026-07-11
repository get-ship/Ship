// pipelines/multi-provider-smoke.ts — per-job source dir for the
// `multi-provider-smoke` pipeline. Migrated from `ship.config.ts`'s inline
// `definePipeline` call (slice 2c, job-spec-manifest-extraction). Byte-same
// payload; inline prompts stay verbatim.
import { definePipeline } from "ship";

definePipeline({
  name: "multi-provider-smoke",
  steps: [
    {
      id: "implement",
      step_specs: [{ id: "implement", agent: "commander", harness: "claude" }],
    },
    {
      id: "verify",
      after: ["implement"],
      step_specs: [{ id: "verify", prompt: "cargo check --workspace 2>&1 | tail -5\n" }],
    },
    {
      id: "review",
      after: ["verify"],
      step_specs: [{ id: "review", agent: "gatekeeper", harness: "claude" }],
      gate: {
        require_changed: ["**/*.rs"],
        forbid_changed: [".artifacts/**"],
      },
    },
  ],
  auto_merge: true,
});
