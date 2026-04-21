-- Migration 0003: drop the legacy event_log table.
--
-- New writers target the `events` table. Grep confirms zero production
-- readers or writers against event_log rows. The `event_log` struct field
-- in kernel_router.rs is an unrelated in-memory Vec and is not affected.
DROP TABLE IF EXISTS event_log;
DROP INDEX IF EXISTS idx_event_workspace;
DROP INDEX IF EXISTS idx_event_session;
DROP INDEX IF EXISTS idx_event_job;
DROP INDEX IF EXISTS idx_event_entity;
