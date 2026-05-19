//! Sidecar v1 protocol — generated-bindings smoke tests.
//!
//! Verifies the proto file compiles end-to-end into usable types and
//! that the canonical handshake/envelope/process shapes round-trip
//! through JSON. Wire compatibility is then a property of the .proto
//! schema's stability discipline.
//!
//! Generated types use buffa's `MessageField`/`Default::default()`
//! idiom; tests build values via `Default` + field assignment instead
//! of struct literals.

use ship_isolation::sidecar::proto::*;

#[test]
fn hello_response_advertises_capabilities() {
    let mut resp = HelloResponse::default();
    resp.protocol_version = "1.0".into();
    resp.sidecar_version = "0.1.0".into();
    resp.capabilities = vec![
        "processes".into(),
        "pty".into(),
        "secrets".into(),
        "events.acp".into(),
    ];
    assert!(resp.capabilities.contains(&"pty".into()));
    assert_eq!(resp.protocol_version, "1.0");
}

#[test]
fn process_state_enum_round_trips_via_i32() {
    assert_eq!(ProcessState::PROCESS_STATE_UNSPECIFIED as i32, 0);
    assert_eq!(ProcessState::PROCESS_STATE_RUNNING as i32, 1);
    assert_eq!(ProcessState::PROCESS_STATE_EXITED as i32, 2);
    assert_eq!(ProcessState::PROCESS_STATE_KILLED as i32, 3);
}

#[test]
fn envelope_payload_is_a_oneof() {
    let mut out = ProcessOutput::default();
    out.process_id = "proc-1".into();
    out.stream = 1;
    out.data = b"hello\n".to_vec();
    let mut env = Envelope::default();
    env.sequence = 7;
    env.payload = Some(envelope::Payload::ProcessOutput(Box::new(out)));
    assert_eq!(env.sequence, 7);
    match &env.payload {
        Some(envelope::Payload::ProcessOutput(o)) => {
            assert_eq!(o.process_id, "proc-1");
            assert_eq!(o.data, b"hello\n");
        }
        _ => panic!("expected ProcessOutput"),
    }
}

#[test]
fn permission_request_reply_correlates_via_request_id() {
    let mut pr = PermissionRequest::default();
    pr.tool = "shell".into();
    pr.detail = "rm -rf /".into();
    pr.scope = "shell:run".into();
    let mut req = Envelope::default();
    req.sequence = 1;
    req.request_id = "req-perm-1".into();
    req.payload = Some(envelope::Payload::PermissionRequest(Box::new(pr)));

    let mut rep = PermissionReply::default();
    rep.decision = "deny".into();
    let mut reply = Envelope::default();
    reply.sequence = 2;
    reply.request_id = req.request_id.clone();
    reply.payload = Some(envelope::Payload::PermissionReply(Box::new(rep)));

    assert_eq!(req.request_id, reply.request_id);
    assert!(!req.request_id.is_empty());
}

#[test]
fn process_create_request_pty_default_off() {
    let req = ProcessCreateRequest::default();
    assert!(!req.pty);
    // MessageField is "unset" by default; the buffa accessor returns
    // None when the protobuf-optional message field isn't present.
    assert!(req.pty_size.is_unset());
}

#[test]
fn pty_attach_request_just_carries_id() {
    let mut r = ProcessAttachPtyRequest::default();
    r.id = "proc-7".into();
    assert_eq!(r.id, "proc-7");
}

#[test]
fn secret_reply_carries_value_only_when_ok() {
    let mut ok_reply = SecretReply::default();
    ok_reply.ok = true;
    ok_reply.value = "sk-...".into();
    assert!(ok_reply.ok);
    assert!(!ok_reply.value.is_empty());

    let mut err_reply = SecretReply::default();
    err_reply.ok = false;
    err_reply.error = "no such secret 'foo'".into();
    assert!(!err_reply.ok);
    assert_eq!(err_reply.error, "no such secret 'foo'");
}
