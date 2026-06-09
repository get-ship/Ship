-- Retire the legacy agent_sessions projection table.
-- All session state is now tracked in canonical_sessions (runtime.db).
-- agent.session.started is still emitted as the spawn trigger for the
-- service.agents actor, but no projection folds those events any more.
DROP TABLE IF EXISTS agent_sessions;
