// collections/systems.ts — per-job source dir for the `systems` collection.
// Migrated from `ship.config.ts`'s inline `defineCollection` call (slice 2c,
// job-spec-manifest-extraction). Byte-same payload; no field changes.
import { defineCollection } from "ship";

defineCollection({
  id: "systems",
  name: "Ship Systems",
  fields: [
    { name: "name", type: "string", required: true, filterable: true },
    {
      name: "layer",
      type: "enum",
      values: ["primitive", "runtime", "interface", "meta"],
      filterable: true,
    },
    {
      name: "status",
      type: "enum",
      values: ["spec", "building", "shipping", "complete"],
      filterable: true,
      default: "spec",
    },
    { name: "summary", type: "string" },
    { name: "current_state", type: "markdown" },
    { name: "desired_state", type: "markdown" },
    { name: "gaps", type: "array" },
    { name: "diagram", type: "markdown" },
    { name: "doc_skill", type: "string", filterable: true },
    { name: "code_paths", type: "array" },
    { name: "depends_on", type: "array", filterable: true },
    { name: "position_x", type: "number" },
    { name: "position_y", type: "number" },
  ],
});
