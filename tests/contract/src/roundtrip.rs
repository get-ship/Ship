//! Contract conformance: roundtrip + registry-metadata assertions.
//!
//! For each event fixture in proto/fixtures/events/:
//!   - Load <event>.json → parse payload → serialize back → assert byte-equal
//!   - Load <event>.envelope.json → same roundtrip
//!   - Assert scope/retention/stability metadata matches the live registry
//!
//! For each op fixture in proto/fixtures/<service>.json:
//!   - Load → parse → serialize back → assert byte-equal (JSON roundtrip)
//!   - Assert that typed methods have a response_ok entry
//!   - For methods with a "request" key: same roundtrip
//!
//! NEEDS_AUTHOR fixtures (those with a `_note` field starting with "NEEDS_AUTHOR")
//! are NOT failed — they are collected and reported at the end as informational.

use ship_contract_tests::{assert_json_roundtrip, fixtures_dir, load_fixture};
use ship_ops_rpc::policy::{is_non_domain_service, op_name, read_policies, snake};
use ship_types::event_registry::{registry, EventScope, Retention, Stability};

// pull in inventory submissions so the registry populates. Additional
// domains that own `register_event!` calls are needed here too — the events
// corpus gate must see the FULL registry, not just ship-substrate's
// residual events. ship-space is deliberately absent: its wire.rs does not
// compile against the current ship-proto SSOT (unrelated drift), so its 3
// event types are invisible here until that's fixed separately.
use agent_def as _;
use ship_account as _;
use ship_artifacts as _;
use ship_auth as _;
use ship_cloud as _;
use ship_collection as _;
use ship_settings as _;
use ship_space as _;
use ship_substrate as _;

// ── Event fixture roundtrip ───────────────────────────────────────────────────

/// Check if a fixture value carries a NEEDS_AUTHOR note (auto-gen failed).
fn is_needs_author(v: &serde_json::Value) -> bool {
    v.get("payload")
        .and_then(|p| p.get("_note"))
        .and_then(|n| n.as_str())
        .map(|s| s.starts_with("NEEDS_AUTHOR"))
        .unwrap_or(false)
}

fn scope_str(s: &EventScope) -> &'static str {
    s.as_str()
}

fn retention_str(r: &Retention) -> &'static str {
    match r {
        Retention::Durable => "durable",
        Retention::Windowed { .. } => "windowed",
        Retention::Ephemeral => "ephemeral",
    }
}

fn stability_str(s: &Stability) -> &'static str {
    match s {
        Stability::Stable => "stable",
        Stability::Deprecated => "deprecated",
        Stability::Experimental => "experimental",
    }
}

#[test]
fn event_fixtures_roundtrip_and_metadata() {
    let dir = fixtures_dir().join("events");
    if !dir.exists() {
        panic!("<PROTO_ROOT>/fixtures/events/ does not exist — PROTO_ROOT is resolved by tests/contract/build.rs from the ship-proto dep (default = the ship-contracts SSOT checkout); set SHIP_PROTO_DIR to override for an alternate layout");
    }

    let registry_map: std::collections::HashMap<&str, _> =
        registry().map(|d| (d.name, d)).collect();

    let mut gaps: Vec<String> = Vec::new();
    let mut needs_author: Vec<String> = Vec::new();

    for def in registry() {
        let safe = def.name.replace('.', "_");

        // ── Payload fixture ───────────────────────────────────────────────────
        let payload_path = dir.join(format!("{safe}.json"));
        if !payload_path.exists() {
            gaps.push(format!("{}: missing payload fixture", def.name));
            continue;
        }
        let payload_fixture = load_fixture(&payload_path);

        if is_needs_author(&payload_fixture) {
            needs_author.push(format!("{}: NEEDS_AUTHOR (skipping roundtrip)", def.name));
        } else {
            // JSON roundtrip
            let rt = assert_json_roundtrip(&format!("{} payload", def.name), &payload_fixture);
            assert_eq!(
                payload_fixture, rt,
                "{}: payload fixture did not survive JSON roundtrip",
                def.name
            );
        }

        // ── Metadata conformance ──────────────────────────────────────────────
        if let Some(meta) = payload_fixture.get("_meta") {
            let fixture_scope = meta.get("scope").and_then(|v| v.as_str()).unwrap_or("");
            let fixture_retention = meta.get("retention").and_then(|v| v.as_str()).unwrap_or("");
            let fixture_stability = meta.get("stability").and_then(|v| v.as_str()).unwrap_or("");

            let reg = registry_map[def.name];
            let expected_scope = scope_str(&reg.primary_scope());
            let expected_retention = retention_str(&reg.retention);
            let expected_stability = stability_str(&reg.stability);

            if fixture_scope != expected_scope {
                gaps.push(format!(
                    "{}: scope drift — fixture={fixture_scope}, registry={expected_scope}",
                    def.name
                ));
            }
            if fixture_retention != expected_retention {
                gaps.push(format!(
                    "{}: retention drift — fixture={fixture_retention}, registry={expected_retention}",
                    def.name
                ));
            }
            if fixture_stability != expected_stability {
                gaps.push(format!(
                    "{}: stability drift — fixture={fixture_stability}, registry={expected_stability}",
                    def.name
                ));
            }
        }

        // ── Envelope fixture ─────────────────────────────────────────────────
        let envelope_path = dir.join(format!("{safe}.envelope.json"));
        if !envelope_path.exists() {
            gaps.push(format!("{}: missing envelope fixture", def.name));
            continue;
        }
        let envelope_fixture = load_fixture(&envelope_path);
        let rt_env = assert_json_roundtrip(&format!("{} envelope", def.name), &envelope_fixture);
        assert_eq!(
            envelope_fixture, rt_env,
            "{}: envelope fixture did not survive JSON roundtrip",
            def.name
        );

        // Assert envelope has the correct event_type field
        let env_event_type = envelope_fixture
            .get("envelope")
            .and_then(|e| e.get("event_type"))
            .and_then(|v| v.as_str())
            .unwrap_or("");
        if env_event_type != def.name {
            gaps.push(format!(
                "{}: envelope.event_type mismatch: got `{env_event_type}`",
                def.name
            ));
        }
    }

    // NEEDS_AUTHOR are informational — not failures.
    if !needs_author.is_empty() {
        eprintln!(
            "contract: {} NEEDS_AUTHOR event(s) — roundtrip skipped, hand-author required:",
            needs_author.len()
        );
        for item in &needs_author {
            eprintln!("  {item}");
        }
    }

    assert!(
        gaps.is_empty(),
        "contract: event fixture gaps ({}):\n  {}",
        gaps.len(),
        gaps.join("\n  ")
    );
}

// ── Event corpus gate: registry↔fixture set equality ──────────────────────────
//
// Charter I-5 / blueprint/subsystems/ship-events.md: every registered event
// type has exactly one payload + one envelope fixture; every fixture
// corresponds to a registered type. No retired-type fixtures survive.
//
// ship-space's 3 event types (space.opened, space.seen, stack.registered) are
// invisible to this binary — ship-space's wire.rs does not compile against
// the current ship-proto SSOT (unrelated drift: SpaceServiceTrait::add vs the
// create/open split), so ship-space cannot be linked here. No fixtures exist
// for these 3 types in the corpus either, so this gate neither false-flags
// nor silently passes them — they're simply outside its view until the
// ship-space blocker is fixed.
#[test]
fn event_corpus_set_equality() {
    let dir = fixtures_dir().join("events");
    if !dir.exists() {
        panic!("<PROTO_ROOT>/fixtures/events/ does not exist — see event_fixtures_roundtrip_and_metadata for PROTO_ROOT resolution");
    }

    let expected: std::collections::BTreeSet<String> =
        registry().map(|d| d.name.replace('.', "_")).collect();

    let mut actual_payload: std::collections::BTreeSet<String> = Default::default();
    let mut actual_envelope: std::collections::BTreeSet<String> = Default::default();

    for entry in
        std::fs::read_dir(&dir).unwrap_or_else(|e| panic!("cannot read {}: {e}", dir.display()))
    {
        let file_name = entry
            .expect("dir entry")
            .file_name()
            .to_string_lossy()
            .into_owned();
        if let Some(stem) = file_name.strip_suffix(".envelope.json") {
            actual_envelope.insert(stem.to_string());
        } else if let Some(stem) = file_name.strip_suffix(".json") {
            actual_payload.insert(stem.to_string());
        }
    }

    let mut gaps: Vec<String> = Vec::new();
    for orphan in actual_payload.difference(&expected) {
        gaps.push(format!(
            "{orphan}.json: orphaned payload fixture — no registered event by this name (retired type never pruned?)"
        ));
    }
    for orphan in actual_envelope.difference(&expected) {
        gaps.push(format!(
            "{orphan}.envelope.json: orphaned envelope fixture — no registered event by this name (retired type never pruned?)"
        ));
    }
    for missing in expected.difference(&actual_payload) {
        gaps.push(format!(
            "{missing}: registered event has no payload fixture"
        ));
    }
    for missing in expected.difference(&actual_envelope) {
        gaps.push(format!(
            "{missing}: registered event has no envelope fixture"
        ));
    }

    assert!(
        gaps.is_empty(),
        "contract: event corpus set mismatch ({}):\n  {}",
        gaps.len(),
        gaps.join("\n  ")
    );
}

// ── Event corpus gate: envelope metadata matches registry ─────────────────────

#[test]
fn event_envelope_metadata_matches_registry() {
    let dir = fixtures_dir().join("events");
    let mut gaps: Vec<String> = Vec::new();

    for def in registry() {
        let safe = def.name.replace('.', "_");
        let envelope_path = dir.join(format!("{safe}.envelope.json"));
        if !envelope_path.exists() {
            continue; // reported by event_corpus_set_equality
        }
        let envelope_fixture = load_fixture(&envelope_path);
        let meta = match envelope_fixture.get("_meta") {
            Some(m) => m,
            None => {
                gaps.push(format!("{}: envelope fixture missing _meta", def.name));
                continue;
            }
        };

        let fixture_scope = meta.get("scope").and_then(|v| v.as_str()).unwrap_or("");
        let fixture_retention = meta.get("retention").and_then(|v| v.as_str()).unwrap_or("");
        let fixture_stability = meta.get("stability").and_then(|v| v.as_str()).unwrap_or("");

        let expected_scope = scope_str(&def.primary_scope());
        let expected_retention = retention_str(&def.retention);
        let expected_stability = stability_str(&def.stability);

        if fixture_scope != expected_scope {
            gaps.push(format!(
                "{}: envelope scope drift — fixture={fixture_scope}, registry={expected_scope}",
                def.name
            ));
        }
        if fixture_retention != expected_retention {
            gaps.push(format!(
                "{}: envelope retention drift — fixture={fixture_retention}, registry={expected_retention}",
                def.name
            ));
        }
        if fixture_stability != expected_stability {
            gaps.push(format!(
                "{}: envelope stability drift — fixture={fixture_stability}, registry={expected_stability}",
                def.name
            ));
        }
    }

    assert!(
        gaps.is_empty(),
        "contract: event envelope metadata gaps ({}):\n  {}",
        gaps.len(),
        gaps.join("\n  ")
    );
}

// ── Event corpus gate: payload replay through current structs ─────────────────
//
// A payload change that breaks an existing fixture fails the build — this is
// what makes `Stability::Stable` mean something (charter I-5).

#[test]
fn event_payload_replay_through_current_structs() {
    let dir = fixtures_dir().join("events");
    let mut failures: Vec<String> = Vec::new();
    let mut replayed = 0usize;
    let mut skipped_needs_author = 0usize;

    for def in registry() {
        let safe = def.name.replace('.', "_");
        let payload_path = dir.join(format!("{safe}.json"));
        if !payload_path.exists() {
            continue; // reported by event_corpus_set_equality
        }
        let fixture = load_fixture(&payload_path);
        if is_needs_author(&fixture) {
            skipped_needs_author += 1;
            continue;
        }
        let payload = match fixture.get("payload") {
            Some(p) => p,
            None => {
                failures.push(format!("{}: fixture missing `payload` key", def.name));
                continue;
            }
        };
        let payload_json = serde_json::to_string(payload)
            .unwrap_or_else(|e| panic!("{}: cannot serialize payload for replay: {e}", def.name));
        if let Err(e) = (def.validate_payload)(&payload_json) {
            failures.push(format!(
                "{}: payload fixture no longer deserializes into `{}` — {e}",
                def.name, def.payload_type_name
            ));
        }
        replayed += 1;
    }

    eprintln!(
        "contract: replayed {replayed} event payload fixture(s) through current structs ({skipped_needs_author} NEEDS_AUTHOR skipped)"
    );
    assert!(
        failures.is_empty(),
        "contract: payload replay failures ({}):\n  {}",
        failures.len(),
        failures.join("\n  ")
    );
}

// ── Op fixture roundtrip ──────────────────────────────────────────────────────

#[test]
fn op_fixtures_roundtrip() {
    let fixtures_dir = fixtures_dir();
    let policies = read_policies();

    let mut gaps: Vec<String> = Vec::new();

    // Collect per-service files that exist
    let mut seen_services = std::collections::BTreeSet::new();
    for p in &policies {
        if is_non_domain_service(&p.service) {
            continue;
        }
        if p.output_type.is_empty() || p.output_type == "JsonResponse" {
            continue;
        }
        let svc = snake(&p.service);
        if seen_services.contains(&svc) {
            continue;
        }
        seen_services.insert(svc.clone());

        let path = fixtures_dir.join(format!("{svc}.json"));
        if !path.exists() {
            gaps.push(format!("{svc}: service fixture file missing",));
            continue;
        }
        let file_val = load_fixture(&path);
        let rt = assert_json_roundtrip(&svc, &file_val);
        assert_eq!(file_val, rt, "{svc}.json did not survive JSON roundtrip");
    }

    // Check per-method entries exist and have response_ok
    for p in &policies {
        if is_non_domain_service(&p.service) {
            continue;
        }
        // DeviceService is unfinished (#130) — its rpcs are typed but have no
        // validators/fixtures yet (gen-fixtures skips it via is_non_fixture_service).
        // Skip until the device-window service lands; then this re-arms.
        if p.service == "DeviceService" {
            continue;
        }
        if p.output_type.is_empty() || p.output_type == "JsonResponse" {
            continue;
        }
        let svc = snake(&p.service);
        let path = fixtures_dir.join(format!("{svc}.json"));
        if !path.exists() {
            continue;
        } // already reported above

        let file_val = load_fixture(&path);
        let method_entry = file_val.get(&p.method);
        match method_entry {
            None => gaps.push(format!(
                "{}: no entry in {svc}.json",
                op_name(&p.service, &p.method)
            )),
            Some(entry) => {
                if entry.get("response_ok").is_none() {
                    gaps.push(format!(
                        "{}: missing response_ok",
                        op_name(&p.service, &p.method)
                    ));
                }
                // Request fixture: only check if args exist (no-arg ops legitimately skip this)
                if !p.args.is_empty() && entry.get("request").is_none() {
                    gaps.push(format!(
                        "{}: missing request stub (has {} arg(s))",
                        op_name(&p.service, &p.method),
                        p.args.len()
                    ));
                }
            }
        }
    }

    assert!(
        gaps.is_empty(),
        "contract: op fixture gaps ({}):\n  {}",
        gaps.len(),
        gaps.join("\n  ")
    );
}
