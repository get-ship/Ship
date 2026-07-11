// collections/architecture-observations.ts — per-job source dir for the
// `architecture-observations` collection. Migrated from `ship.config.ts`'s
// inline `defineCollection` call (slice 2c, job-spec-manifest-extraction).
// Byte-same payload; no field changes.
import { defineCollection } from "ship";

defineCollection({
  id: "architecture-observations",
  name: "Architecture Observations",
  fields: [
    { name: "id", type: "string", required: true, filterable: true },
    { name: "module", type: "string", required: true, filterable: true },
    {
      name: "layer",
      type: "enum",
      values: ["L0", "L1", "L2", "shared", "apps"],
      filterable: true,
    },
    { name: "tag", type: "string", required: true, filterable: true },
    {
      name: "tag_status",
      type: "enum",
      values: ["candidate", "confirmed", "promoted"],
      filterable: true,
    },
    {
      name: "severity",
      type: "enum",
      values: ["info", "low", "medium", "high", "critical"],
      filterable: true,
      sortable: true,
    },
    { name: "citation_file", type: "string" },
    { name: "citation_line", type: "string" },
    { name: "description", type: "markdown", required: true },
    {
      name: "status",
      type: "enum",
      values: [
        "pending",
        "under-review",
        "signed-off",
        "superseded",
        "closed",
      ],
      filterable: true,
      default: "pending",
    },
    { name: "audit_doc", type: "string" },
    { name: "task_id", type: "string" },
    { name: "pass_date", type: "string" },
    { name: "notes", type: "markdown" },
  ],
});
