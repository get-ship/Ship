// pipelines/smoke.ts — per-job source dir for the `smoke` pipeline.
// Migrated from `ship.config.ts`'s inline `definePipeline` call (slice 2c,
// job-spec-manifest-extraction). Byte-same payload; no field changes;
// inline bash step prompts stay verbatim — `PipelineStepSpec` has no
// `script-path` field, so file-extraction would invent schema (the
// job spec forbids inventing schema; the prompts live inline in the
// moved file as the spec directs).
import { definePipeline } from "ship";

definePipeline({
  name: "smoke",
  steps: [
    {
      id: "smoke-step-1",
      step_specs: [{ id: "smoke-step-1", prompt: "echo \"smoke step-1: scratch work\" > smoke.txt\ngit add smoke.txt\ngit -c user.name=smoke -c user.email=smoke@ship.dev commit -m \"smoke: step-1 creates smoke.txt\"\n" }],
    },
    {
      id: "smoke-step-2",
      step_specs: [{ id: "smoke-step-2", prompt: "echo \"smoke step-2: modify\" >> smoke.txt\ngit -c user.name=smoke -c user.email=smoke@ship.dev commit -am \"smoke: step-2 appends to smoke.txt\"\n" }],
    },
    {
      id: "smoke-step-3",
      step_specs: [{ id: "smoke-step-3", prompt: "echo \"smoke step-3: verify\" >&2\ntest -f smoke.txt && grep -q \"step-2\" smoke.txt\n" }],
    },
  ],
  auto_merge: false,
});
