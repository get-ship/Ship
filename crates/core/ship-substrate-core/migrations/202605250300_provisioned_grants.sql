-- provisioned_grants: the per-agent approved grant baseline the mint path reads.
--
-- One row per agent_id (latest write wins). Materialized from
-- `agent.grants.provisioned` events. Seeded at install (source='install',
-- from the agent's resolved skill grants) and rewritten on operator approval
-- (source='operator'). The token-mint path reads this baseline instead of
-- deriving grants from the role string; the gate then enforces against the
-- minted session's grants.
--
-- See .artifacts/capability-provisioning-and-approval.md +
-- .artifacts/grant-provisioning-85-plan.md.

CREATE TABLE IF NOT EXISTS provisioned_grants (
  agent_id     TEXT PRIMARY KEY NOT NULL,
  grants_json  TEXT NOT NULL,          -- JSON array of restricted-domain grant ids
  source       TEXT NOT NULL,          -- 'install' | 'operator'
  approved_by  TEXT,                   -- operator/user id for 'operator' writes
  updated_at   TEXT NOT NULL           -- ISO-8601
);
