// collections/ops-gap.ts — per-job source dir for the `ops-gap` collection.
// Migrated from `ship.config.ts`'s inline `defineCollection` call (slice 2c,
// job-spec-manifest-extraction). Byte-same payload; no field changes.
import { defineCollection } from "ship";

defineCollection({
  id: "ops-gap",
  name: "Ops Catalog Gap",
  fields: [
    {
      name: "service",
      type: "string",
      required: true,
      filterable: true,
    },
    {
      name: "op_name",
      type: "string",
      required: true,
      filterable: true,
    },
    {
      name: "substrate_fn",
      type: "string",
      required: true,
    },
    {
      name: "audience",
      type: "enum",
      values: ["operator", "agent", "both"],
      filterable: true,
    },
    {
      name: "capability",
      type: "enum",
      values: ["C0", "C1", "C2", "C2_ELEVATED", "C3"],
      filterable: true,
    },
    {
      name: "priority",
      type: "enum",
      values: ["P0", "P1", "P2", "deferred"],
      filterable: true,
      sortable: true,
    },
    {
      name: "status",
      type: "enum",
      values: [
        "missing",
        "scaffolded",
        "implemented",
        "rejected-no-wire",
        "in-flight",
      ],
      filterable: true,
      default: "missing",
    },
    { name: "rationale", type: "markdown" },
    { name: "audit_source", type: "string" },
    { name: "consumer_demand", type: "markdown" },
    { name: "dependencies", type: "string" },
  ],
});
