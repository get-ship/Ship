// pipelines/view-session-e2e.ts — per-job source dir for the
// `view-session-e2e` pipeline. Migrated from `ship.config.ts`'s inline
// `definePipeline` call (slice 2c, job-spec-manifest-extraction).
//
// ## Inline-prompt decision (script-path field)
//
// The job spec asks: extract the inline bash step to a separate
// `pipelines/view-session-e2e.sh` file IF the step schema has a
// `script-path` field; if not, leave the string inline. Per
// `crates/core/compiler/src/types/pipeline.rs` (the only
// `PipelineStepSpec` definition), the schema has NO `script-path`
// field — only `id`, `agent`, `harness`, `prompt`. Inventing one would
// be a schema change, which the spec forbids. Decision: keep the
// inline `prompt` strings verbatim in this file. Reported in the
// slice 2c gate output.
import { definePipeline } from "ship";

definePipeline({
  name: "view-session-e2e",
  steps: [
    {
      id: "open",
      step_specs: [{ id: "open", prompt: "set -euo pipefail\n: \"${SHIP_DAEMON_TOKEN:?SHIP_DAEMON_TOKEN must be set}\"\nurl=\"${SHIP_DAEMON_URL:-http://127.0.0.1:9315}\"\n: \"${SHIP_MISSION_SCRATCH:=$(pwd)}\"\n\nresp=$(curl -fsS -X POST \"$url/api/view-sessions\" \\\n  -H \"Authorization: Bearer $SHIP_DAEMON_TOKEN\" \\\n  -H \"x-ship-agent-id: pipeline.e2e\" \\\n  -H \"content-type: application/json\" \\\n  -d '{\"template\":\"canvas/blank\",\"title\":\"e2e\"}')\necho \"$resp\" | tee \"$SHIP_MISSION_SCRATCH/view-session-open.json\"\nview_id=$(echo \"$resp\" | sed -n 's/.*\"view_id\":\"\\([^\"]*\\)\".*/\\1/p')\necho \"$view_id\" > \"$SHIP_MISSION_SCRATCH/view-session-id\"\n" }],
    },
    {
      id: "subscribe-and-patch",
      step_specs: [{ id: "subscribe-and-patch", prompt: "set -euo pipefail\n: \"${SHIP_DAEMON_TOKEN:?SHIP_DAEMON_TOKEN must be set}\"\nurl=\"${SHIP_DAEMON_URL:-http://127.0.0.1:9315}\"\n: \"${SHIP_MISSION_SCRATCH:=$(pwd)}\"\nview_id=$(cat \"$SHIP_MISSION_SCRATCH/view-session-id\")\n\n# Subscribe in the background, capture SSE body, look for the beacon.\n( timeout 5 curl -sS -N \"$url/api/view-sessions/$view_id\" \\\n    -H \"Authorization: Bearer $SHIP_DAEMON_TOKEN\" \\\n    -H \"x-ship-agent-id: pipeline.e2e\" \\\n    > \"$SHIP_MISSION_SCRATCH/view-session-sse.log\" ) &\nsub_pid=$!\nsleep 1\n\ncurl -fsS -X POST \"$url/api/view-sessions/$view_id/patch\" \\\n  -H \"Authorization: Bearer $SHIP_DAEMON_TOKEN\" \\\n  -H \"x-ship-agent-id: pipeline.e2e\" \\\n  -H \"content-type: application/json\" \\\n  -d '{\"op\":\"append\",\"node\":{\"kind\":\"text\",\"text\":\"hello\"}}'\n\nwait \"$sub_pid\" || true\ngrep -q 'event: beacon' \"$SHIP_MISSION_SCRATCH/view-session-sse.log\" \\\n  || { echo \"no beacon in SSE log\"; cat \"$SHIP_MISSION_SCRATCH/view-session-sse.log\"; exit 1; }\n" }],
    },
    {
      id: "close",
      step_specs: [{ id: "close", prompt: "set -euo pipefail\n: \"${SHIP_DAEMON_TOKEN:?SHIP_DAEMON_TOKEN must be set}\"\nurl=\"${SHIP_DAEMON_URL:-http://127.0.0.1:9315}\"\n: \"${SHIP_MISSION_SCRATCH:=$(pwd)}\"\nview_id=$(cat \"$SHIP_MISSION_SCRATCH/view-session-id\")\n\ncurl -fsS -X DELETE \"$url/api/view-sessions/$view_id\" \\\n  -H \"Authorization: Bearer $SHIP_DAEMON_TOKEN\" \\\n  -H \"x-ship-agent-id: pipeline.e2e\"\n\n# Owner-validation smoke: a different agent id must get 403 on close.\nstatus=$(curl -s -o /dev/null -w '%{http_code}' \\\n  -X DELETE \"$url/api/view-sessions/$view_id\" \\\n  -H \"Authorization: Bearer $SHIP_DAEMON_TOKEN\" \\\n  -H \"x-ship-agent-id: someone.else\")\ntest \"$status\" = \"404\" \\\n  || { echo \"close: expected 404 after prior close, got $status\"; exit 1; }\n" }],
    },
  ],
  auto_merge: false,
});
