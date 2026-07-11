// pipelines/shell-smoke.ts — per-job source dir for the `shell-smoke` pipeline.
// Migrated from `ship.config.ts`'s inline `definePipeline` call (slice 2c,
// job-spec-manifest-extraction). Byte-same payload; inline prompts stay
// verbatim (no `script-path` field on `PipelineStepSpec`).
import { definePipeline } from "ship";

definePipeline({
  name: "shell-smoke",
  steps: [
    {
      id: "one",
      step_specs: [{ id: "one", prompt: "echo \"step 1\"\necho \"step 1\" > smoke.txt\ngit add smoke.txt\ngit -c user.name=smoke -c user.email=smoke@ship.dev commit -m \"smoke step 1\"\n" }],
    },
    {
      id: "two",
      after: ["one"],
      step_specs: [{ id: "two", prompt: "echo \"step 2 appended\" >> smoke.txt\ngit -c user.name=smoke -c user.email=smoke@ship.dev commit -am \"smoke step 2\"\n" }],
    },
    {
      id: "three",
      after: ["two"],
      step_specs: [{ id: "three", prompt: "test -f smoke.txt && grep -q \"step 2 appended\" smoke.txt\n" }],
    },
  ],
  auto_merge: false,
});
