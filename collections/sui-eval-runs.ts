// collections/sui-eval-runs.ts — per-job source dir for the `sui-eval-runs`
// collection. Migrated from `ship.config.ts`'s inline `defineCollection`
// call (slice 2c, job-spec-manifest-extraction). Byte-same payload; no field changes.
import { defineCollection } from "ship";

defineCollection({
  id: "sui-eval-runs",
  name: "SUI Eval Runs",
  fields: [
    { name: "run_id", type: "string", required: true, filterable: true },
    { name: "lane_id", type: "string", required: true, filterable: true },
    { name: "model", type: "string", filterable: true },
    { name: "harness", type: "string", filterable: true },
    { name: "composite", type: "number", filterable: true },
    { name: "compile_passed", type: "boolean", filterable: true },
    { name: "lint_errors", type: "number" },
    { name: "lint_warnings", type: "number" },
    { name: "worktree", type: "string" },
    { name: "view_files", type: "object" },
    { name: "css_violations", type: "object" },
    { name: "missing_fragments", type: "object" },
    { name: "behavior_passed", type: "number" },
    { name: "behavior_total", type: "number" },
    { name: "behavior_score", type: "number", filterable: true },
  ],
});
