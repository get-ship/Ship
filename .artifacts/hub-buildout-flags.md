# Hub buildout flags

Operator-reviewed signals from hub-surface jobs. Appended by the
agent, surfaced for the operator's review (per the hub-buildout-
charter §Escalation). Read this, not micro-decisions in the diff.

## job/hub-home (2026-06-20)

### `BACKEND-GAP` — `ship.spaces.list()` (list-all-spaces)

- **Surface:** the hub `/` home dashboard's spaces grid
  (`<ship-spaces-hero>`, in `packages/components/ship/home/`).
- **Symptom:** the Connect SDK exposes only
  `ship.spaces.config({ id })` (per-space runtime config — see
  `packages/deck/sdk-client/src/spaces.ts` and `ship-sdk.ts:246`).
  There is no `ship.spaces.list()` op; the daemon does not broadcast
  the spaces registry. `/api/v1/projects` is cut (charter §Connect-
  only). Calling it would resurrect a dead endpoint.
- **Surface today:** the home controller (`apps/hub/routes/
  controller.ts`) passes `spacesJson = '[]'`. The component renders
  the empty-state with a "Register a space" CTA; clicking it emits
  `ship-space-add` (host-owned wiring).
- **What the backend team needs to ship:** a Connect op that returns
  the registered spaces (id + name + path). The component's prop
  surface (`spaces={JSON}`) is the contract; the controller switches
  from `derived(() => '[]')` to `live(() => ship.spaces.list())` and
  the empty-state disappears the moment the op lands.
- **Not blocking:** home renders cleanly today; the gap is a
  degradation, not a missing feature path.

No `DESIGN-FORK` or `FRAMEWORK-GAP` from this job.
