//! Supervisor primitive — integration tests against synthetic binaries.

use super::*;
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;

fn tmp_log_dir() -> PathBuf {
    let dir = std::env::temp_dir().join(format!("ship-supervisor-test-{}", ship_substrate::gen_ulid()));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn spawn_pty_minimal() {
    // Spawn `/bin/cat` with no input piped in — it stays alive until
    // we kill it. Exercises PTY open + writer + log file + reader
    // thread + ChildHandle drop semantics.
    let log_dir = tmp_log_dir();
    let profile = ProcessProfile {
        actor_id_hint: None,
        kind: ProcessKind::Shell,
        transport: TransportKind::Pty,
        one_shot: false,
        bin: "/bin/cat".into(),
        args: vec![],
        env: HashMap::new(),
        work_dir: std::env::temp_dir(),
        workspace_id: None,
        agent_id: None,
        harness: None,
        parent_actor_id: None,
        log_dir: Some(log_dir.clone()),
        subscribe_namespaces: vec![],
    };

    let handle = spawn_process(profile).expect("spawn_process failed");

    // Daemon-minted ULID — 26 chars, Crockford alphabet.
    assert_eq!(handle.actor_id.len(), 26);
    assert!(handle.pid().is_some(), "pid populated");
    assert!(handle.is_alive(), "child reachable via /proc");

    // Write input, read echoed output back from broadcast.
    let mut rx = handle.subscribe_output();
    handle.write_input(b"loopback\n").expect("write_input");

    // PTY echo + cat should send something back within a beat.
    let received = std::thread::spawn(move || {
        let deadline = std::time::Instant::now() + Duration::from_secs(2);
        let mut total = Vec::new();
        while std::time::Instant::now() < deadline {
            if let Ok(chunk) = rx.try_recv() {
                total.extend_from_slice(&chunk);
                if !total.is_empty() {
                    break;
                }
            }
            std::thread::sleep(Duration::from_millis(50));
        }
        total
    })
    .join()
    .unwrap();

    assert!(!received.is_empty(), "pty did not echo input");

    handle.stop().expect("stop");

    // Log file was created.
    let logs: Vec<_> = std::fs::read_dir(&log_dir).unwrap().collect();
    assert!(!logs.is_empty(), "log file not created");
}

#[test]
fn spawn_honours_ulid_hint() {
    let hint = "01ABCDEFGHJKMNPQRSTVWXYZ23"; // 26 chars, valid alphabet
    let profile = ProcessProfile {
        actor_id_hint: Some(hint.to_string()),
        kind: ProcessKind::Shell,
        transport: TransportKind::Pty,
        one_shot: false,
        bin: "/bin/cat".into(),
        args: vec![],
        env: HashMap::new(),
        work_dir: std::env::temp_dir(),
        workspace_id: None,
        agent_id: None,
        harness: None,
        parent_actor_id: None,
        log_dir: Some(tmp_log_dir()),
        subscribe_namespaces: vec![],
    };
    let handle = spawn_process(profile).expect("spawn");
    assert_eq!(handle.actor_id, hint);
    handle.stop().unwrap();
}

#[test]
fn spawn_rejects_non_ulid_hint() {
    // Synthesized "agent.X.Y" style ids get a fresh ULID instead.
    let profile = ProcessProfile {
        actor_id_hint: Some("agent.synth.feat-x".to_string()),
        kind: ProcessKind::Shell,
        transport: TransportKind::Pty,
        one_shot: false,
        bin: "/bin/cat".into(),
        args: vec![],
        env: HashMap::new(),
        work_dir: std::env::temp_dir(),
        workspace_id: None,
        agent_id: None,
        harness: None,
        parent_actor_id: None,
        log_dir: Some(tmp_log_dir()),
        subscribe_namespaces: vec![],
    };
    let handle = spawn_process(profile).expect("spawn");
    assert_ne!(handle.actor_id, "agent.synth.feat-x");
    assert_eq!(handle.actor_id.len(), 26);
    handle.stop().unwrap();
}

#[test]
#[ignore = "timing-sensitive: /bin/echo output capture races with CI scheduler"]
fn spawn_headless_drains_stdout_and_exits() {
    // /bin/echo writes to stdout, exits cleanly. Headless transport
    // captures bytes on the broadcast and the child exits within a
    // beat. Exercises stdout reader thread + ChildProcess::try_wait.
    let log_dir = tmp_log_dir();
    let profile = ProcessProfile {
        actor_id_hint: None,
        kind: ProcessKind::Agent,
        transport: TransportKind::Headless,
        one_shot: true,
        bin: "/bin/echo".into(),
        args: vec!["hello-headless".into()],
        env: HashMap::new(),
        work_dir: std::env::temp_dir(),
        workspace_id: None,
        agent_id: None,
        harness: None,
        parent_actor_id: None,
        log_dir: Some(log_dir.clone()),
        subscribe_namespaces: vec![],
    };

    let handle = spawn_process(profile).expect("spawn_process failed");
    assert_eq!(handle.actor_id.len(), 26);
    assert!(handle.pid().is_some());

    let mut rx = handle.subscribe_output();
    let received = std::thread::spawn(move || {
        let deadline = std::time::Instant::now() + Duration::from_secs(2);
        let mut total = Vec::new();
        while std::time::Instant::now() < deadline {
            if let Ok(chunk) = rx.try_recv() {
                total.extend_from_slice(&chunk);
                if !total.is_empty() {
                    break;
                }
            }
            std::thread::sleep(Duration::from_millis(50));
        }
        total
    })
    .join()
    .unwrap();

    assert!(
        String::from_utf8_lossy(&received).contains("hello-headless"),
        "expected 'hello-headless' in headless stdout, got {:?}",
        String::from_utf8_lossy(&received)
    );

    // Headless write_input should fail — no stdin pipe.
    assert!(handle.write_input(b"x").is_err());
}

#[test]
fn stop_terminates_child() {
    let profile = ProcessProfile {
        actor_id_hint: None,
        kind: ProcessKind::Shell,
        transport: TransportKind::Pty,
        one_shot: false,
        bin: "/bin/sh".into(),
        args: vec!["-c".into(), "sleep 30".into()],
        env: HashMap::new(),
        work_dir: std::env::temp_dir(),
        workspace_id: None,
        agent_id: None,
        harness: None,
        parent_actor_id: None,
        log_dir: Some(tmp_log_dir()),
        subscribe_namespaces: vec![],
    };
    let handle = spawn_process(profile).expect("spawn");
    let pid = handle.pid().expect("pid");
    assert!(handle.is_alive());

    handle.stop().expect("stop");

    // /proc/{pid} disappears after the kill propagates.
    let mut alive_after_stop = true;
    for _ in 0..20 {
        if std::fs::metadata(format!("/proc/{pid}")).is_err() {
            alive_after_stop = false;
            break;
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    assert!(!alive_after_stop, "child still alive after stop()");
}
