// triggers/log-failed-jobs.ts — per-job source dir for the `log-failed-jobs`
// trigger. Migrated from `ship.config.ts`'s inline `defineTrigger` call
// (slice 2c, job-spec-manifest-extraction). Byte-same payload; no field changes.
import { defineTrigger } from "ship";

defineTrigger({
  id: "log-failed-jobs",
  trigger: { event: "pipeline.run.failed" },
  actions: [
    {
      type: "event.emit",
      event_type: "trigger.job_failed_notice",
      payload: {
        job_id: "{{ event.entity_id }}",
        triggered_by: "{{ event.id }}",
      },
    },
  ],
});
