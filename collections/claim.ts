// collections/claim.ts — per-job source dir for the `claim` collection.
// Migrated from `ship.config.ts`'s inline `defineCollection` call (slice 2c,
// job-spec-manifest-extraction). Byte-same payload; no field changes.
import { defineCollection } from "ship";

defineCollection({
  id: "claim",
  name: "Claims",
  fields: [
    {
      name: "kind",
      type: "enum",
      values: ["bug", "request", "observation", "hitl"],
      required: true,
      filterable: true,
    },
    {
      name: "by",
      type: "enum",
      values: ["user", "agent", "external"],
      required: true,
      filterable: true,
    },
    {
      name: "target",
      type: "string",
      required: true,
      filterable: true,
    },
    {
      name: "version",
      type: "string",
      required: true,
      filterable: true,
    },
    { name: "body", type: "string", required: true },
    {
      name: "status",
      type: "enum",
      values: ["open", "reconciled", "archived"],
      required: true,
      filterable: true,
      default: "open",
    },
    {
      name: "claimed_at",
      type: "number",
      required: true,
      filterable: true,
      sortable: true,
    },
    { name: "archived_at", type: "string" },
    { name: "reconciliation", type: "object" },
  ],
});
