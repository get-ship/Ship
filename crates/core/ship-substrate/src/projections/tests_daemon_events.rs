//! Round-trip tests for the `daemon.*` event vocabulary.
//!
//! Slice 2 emits these events; slice 4 will project them into a
//! `DaemonRecord`. Until then we only verify the payloads serialize
//! through serde and the platform.db append/read path without losing
//! shape.

#[cfg(test)]
mod tests {
    use crate::db::ensure_db;
    use crate::events::daemon::{
        CrashedPayload, PidfileStaleTakenPayload, PortBindFailedPayload,
        ShutdownCompletedPayload, StartupCompletedPayload,
        event_types as daemon_events,
    };
    use crate::events::{EventStore, SqliteEventStore};
    use crate::events::filter::EventFilter;
    use ship_types::envelope::EventEnvelope;

    fn setup() -> tempfile::TempDir {
        let tmp = tempfile::tempdir().unwrap();
        crate::testutil::init_project_minimal(tmp.path().to_path_buf()).unwrap();
        ensure_db().unwrap();
        tmp
    }

    #[test]
    fn startup_completed_serializes_correctly() {
        let _tmp = setup();
        let payload = StartupCompletedPayload {
            pid: 4242,
            port: 9315,
            version: "0.2.0".into(),
            commit_sha: "deadbeef".into(),
            event_count_at_startup: 17,
            latest_event_id: Some("01HZA000000000000000000000".into()),
            init_duration_ms: 380,
        };
        let envelope = EventEnvelope::new(
            daemon_events::DAEMON_STARTUP_COMPLETED,
            "daemon",
            &payload,
        )
        .unwrap()
        .with_actor_id("daemon");

        let store = SqliteEventStore::new().unwrap();
        store.append(&envelope).unwrap();

        let read = store
            .query(&EventFilter {
                event_type: Some(daemon_events::DAEMON_STARTUP_COMPLETED.into()),
                ..Default::default()
            })
            .unwrap();
        assert_eq!(read.len(), 1);
        let decoded: StartupCompletedPayload =
            serde_json::from_str(&read[0].payload_json).unwrap();
        assert_eq!(decoded.pid, 4242);
        assert_eq!(decoded.port, 9315);
        assert_eq!(decoded.event_count_at_startup, 17);
        assert_eq!(decoded.init_duration_ms, 380);
        assert_eq!(decoded.latest_event_id.as_deref(), Some("01HZA000000000000000000000"));
    }

    #[test]
    fn crashed_payload_handles_signal_or_exit_code() {
        let _tmp = setup();
        let store = SqliteEventStore::new().unwrap();

        // Variant A: clean non-zero exit code, no signal.
        let exit_only = CrashedPayload {
            pid: 1,
            uptime_secs: 10,
            exit_code: Some(2),
            signal: None,
            last_known_event_id: None,
            restart_will_be_attempted: true,
        };
        let env_a =
            EventEnvelope::new(daemon_events::DAEMON_CRASHED, "daemon", &exit_only).unwrap();
        store.append(&env_a).unwrap();

        // Variant B: killed by signal, no exit code.
        let signal_only = CrashedPayload {
            pid: 2,
            uptime_secs: 99,
            exit_code: None,
            signal: Some("SIGSEGV".into()),
            last_known_event_id: Some("evt-9".into()),
            restart_will_be_attempted: false,
        };
        let env_b =
            EventEnvelope::new(daemon_events::DAEMON_CRASHED, "daemon", &signal_only).unwrap();
        store.append(&env_b).unwrap();

        let read = store
            .query(&EventFilter {
                event_type: Some(daemon_events::DAEMON_CRASHED.into()),
                ..Default::default()
            })
            .unwrap();
        assert_eq!(read.len(), 2);
        let mut by_pid: Vec<CrashedPayload> = read
            .iter()
            .map(|e| serde_json::from_str(&e.payload_json).unwrap())
            .collect();
        by_pid.sort_by_key(|p| p.pid);
        assert!(by_pid[0].signal.is_none() && by_pid[0].exit_code == Some(2));
        assert!(by_pid[1].exit_code.is_none() && by_pid[1].signal.as_deref() == Some("SIGSEGV"));
        assert!(by_pid[0].restart_will_be_attempted);
        assert!(!by_pid[1].restart_will_be_attempted);
    }

    #[test]
    fn pidfile_stale_taken_carries_prior_record() {
        let _tmp = setup();
        let payload = PidfileStaleTakenPayload {
            new_pid: 1234,
            prior_pid: 9999,
            prior_started_at: "2026-04-15T10:11:12Z".into(),
            prior_version: "0.1.9".into(),
        };
        let env = EventEnvelope::new(
            daemon_events::DAEMON_PIDFILE_STALE_TAKEN,
            "daemon",
            &payload,
        )
        .unwrap();

        let store = SqliteEventStore::new().unwrap();
        store.append(&env).unwrap();

        let read = store
            .query(&EventFilter {
                event_type: Some(daemon_events::DAEMON_PIDFILE_STALE_TAKEN.into()),
                ..Default::default()
            })
            .unwrap();
        assert_eq!(read.len(), 1);
        let decoded: PidfileStaleTakenPayload =
            serde_json::from_str(&read[0].payload_json).unwrap();
        assert_eq!(decoded.new_pid, 1234);
        assert_eq!(decoded.prior_pid, 9999);
        assert_eq!(decoded.prior_started_at, "2026-04-15T10:11:12Z");
        assert_eq!(decoded.prior_version, "0.1.9");
    }

    #[test]
    fn shutdown_and_port_bind_failed_round_trip() {
        let _tmp = setup();
        let store = SqliteEventStore::new().unwrap();

        let shutdown = ShutdownCompletedPayload {
            pid: 7,
            uptime_secs: 3600,
            agents_drained: 4,
            trigger: "sigterm".into(),
        };
        let bind = PortBindFailedPayload {
            port: 9315,
            already_bound_by_pid: Some(8001),
            error: "Address already in use".into(),
        };
        store
            .append(
                &EventEnvelope::new(daemon_events::DAEMON_SHUTDOWN_COMPLETED, "daemon", &shutdown)
                    .unwrap(),
            )
            .unwrap();
        store
            .append(
                &EventEnvelope::new(daemon_events::DAEMON_PORT_BIND_FAILED, "daemon", &bind)
                    .unwrap(),
            )
            .unwrap();

        let shutdown_back: ShutdownCompletedPayload = serde_json::from_str(
            &store
                .query(&EventFilter {
                    event_type: Some(daemon_events::DAEMON_SHUTDOWN_COMPLETED.into()),
                    ..Default::default()
                })
                .unwrap()[0]
                .payload_json,
        )
        .unwrap();
        let bind_back: PortBindFailedPayload = serde_json::from_str(
            &store
                .query(&EventFilter {
                    event_type: Some(daemon_events::DAEMON_PORT_BIND_FAILED.into()),
                    ..Default::default()
                })
                .unwrap()[0]
                .payload_json,
        )
        .unwrap();
        assert_eq!(shutdown_back.agents_drained, 4);
        assert_eq!(shutdown_back.trigger, "sigterm");
        assert_eq!(bind_back.already_bound_by_pid, Some(8001));
    }
}
