//! Scenario 3 — identifier injection torture.
//!
//! Master property 3: no unsafe identifier or query ever reaches SQL.
//! `is_safe_identifier` (schema.rs, gates schema/field names at
//! `validate_schema_doc` time) and `is_safe_ident` (ship_substrate, gates
//! every identifier that actually reaches a DDL/DML string) must both hold
//! the line. This file torches both with quotes, semicolons, `--`, unicode
//! homoglyphs, SQL keywords, and path traversal, and proves end-to-end that
//! nothing unsafe reaches a materialized table/column.

use std::sync::Arc;

use serde_json::json;
use ship_collection::projection::SchemaRegistry;
use ship_collection::schema::is_safe_identifier;
use ship_collection::service::CollectionService;
use ship_substrate::data::DataService;
use ship_substrate::events::new_isolated_for_test_with;
use ship_substrate::is_safe_ident;
use ship_substrate::{KernelLog, SingleBackendDataStore, SpaceContext, SpaceId};

mod common;

fn make_kernel(tmp: &tempfile::TempDir) -> Arc<KernelLog> {
    let ship_dir = tmp.path().join(".ship");
    std::fs::create_dir_all(&ship_dir).unwrap();
    let router =
        new_isolated_for_test_with(ship_dir, common::collection_projection_factory()).unwrap();
    Arc::new(KernelLog::new(router))
}

fn make_ctx(tmp: &tempfile::TempDir, kernel: Arc<KernelLog>) -> Arc<SpaceContext> {
    let ship_dir = tmp.path().join(".ship");
    ship_substrate::testutil::register_space(&ship_dir, "test").unwrap();
    let space_db = ship_substrate::db::space_db_path(&ship_dir, "test");
    let svc = DataService::open_at(&space_db).unwrap();
    let store = Arc::new(SingleBackendDataStore::new(svc, "test"));
    Arc::new(SpaceContext::new(SpaceId::new("test"), store, kernel))
}

fn make_service(ctx: Arc<SpaceContext>) -> CollectionService {
    CollectionService {
        ctx,
        schemas: Arc::new(SchemaRegistry::new()),
    }
}

// ── is_safe_ident (the strict, DDL-facing gate) — the payload set ────────────

const MALICIOUS_IDENTS: &[&str] = &[
    // quotes / escaping
    "a`b",
    "a\"b",
    "a'b",
    "a\\b",
    // statement injection
    "x; DROP TABLE items; --",
    "x--comment",
    "x/*comment*/y",
    // SQL keywords (must still be rejected/accepted per grammar rules, not
    // specially blacklisted — but a bare keyword like this used as an ident
    // should not be able to break out of backticks)
    "DROP TABLE items",
    "SELECT * FROM x",
    // path traversal look-alikes
    "../../etc/passwd",
    "..\\..\\windows",
    "..",
    ".",
    // whitespace / control
    "a b",
    "a\nb",
    "a\tb",
    "a\0b",
    // unicode homoglyphs of ASCII letters (Cyrillic а, е; fullwidth A)
    "\u{0430}dmin",
    "\u{0435}vent",
    "\u{FF21}dmin",
    // empty / overlong
    "",
    // leading digit (SQL identifiers can't start with a digit unquoted)
    "1abc",
    // leading hyphen/dot (would look like a CLI flag / relative path)
    "-rf",
    ".hidden",
];

#[test]
fn is_safe_ident_rejects_every_malicious_payload() {
    for payload in MALICIOUS_IDENTS {
        assert!(
            !is_safe_ident(payload),
            "is_safe_ident must reject {payload:?} but accepted it"
        );
    }
}

#[test]
fn is_safe_ident_accepts_ordinary_identifiers() {
    for ok in ["title", "_private", "field_2", "a", "CamelCase"] {
        assert!(is_safe_ident(ok), "{ok:?} should be a safe identifier");
    }
}

/// FINDING: `ship_collection::schema::is_safe_identifier` (used to gate
/// `SchemaDoc.id` and field names at `validate_schema_doc` time) is a
/// DIFFERENT, looser rule than `ship_substrate::is_safe_ident` (used at
/// `item_projection.rs::check_ident`, the function that actually builds every
/// `ALTER TABLE`/`INSERT`/`UPDATE` DDL/DML string). schema.rs's version:
/// accepts any Unicode alphanumeric char (incl. homoglyphs) plus `-` and `.`,
/// and does not require a non-digit first character or bound length.
/// ship_substrate's version: ASCII-only alphanumeric + `_`, must start with
/// alpha/`_`, max 128 chars. A schema whose field name passes the former but
/// fails the latter (e.g. `"a-b"`, `"a.b"`, `"1abc"`, a Cyrillic homoglyph)
/// is silently ACCEPTED as a valid schema by `create_schema`/`update_schema`
/// (`validate_schema_doc` passes) but then can never be written to: the
/// `collection.created`/`collection.schema.changed` projection's
/// `check_ident` (the strict gate) rejects it when building the `ALTER TABLE
/// ADD COLUMN` statement, and every `create_item`/`update_item` carrying that
/// field hits the same wall in `check_ident` inside `upsert_item`/
/// `update_item`. No SQL injection results (the strict gate still holds at
/// the DDL boundary), but this is a real break of master property 1: the
/// schema accepts a shape the projection can never materialize, so "table
/// matches schema" is false by construction for that field. Demonstrated end
/// to end below.
#[test]
fn schema_is_safe_identifier_is_looser_than_the_ddl_facing_is_safe_ident() {
    let permissive_but_unsafe = ["a-b", "a.b", "1abc", "\u{0430}dmin", "-rf", ".hidden"];
    for name in permissive_but_unsafe {
        assert!(
            is_safe_identifier(name),
            "schema.rs::is_safe_identifier is expected (today) to accept {name:?}"
        );
        assert!(
            !is_safe_ident(name),
            "ship_substrate::is_safe_ident is expected to reject {name:?} — \
             if this now passes, the two gates converged and this test (and \
             the finding below) is stale"
        );
    }
}

/// End-to-end proof of the finding above. `validate_schema_doc` accepts
/// `"foo-bar"` as a field name (schema.rs's loose `is_safe_identifier`), so
/// `create_schema` proceeds to emit `collection.created` — but the SAME-TX
/// `CollectionItemProjection` then runs `check_ident("foo-bar")`
/// (`ship_substrate::is_safe_ident`, the strict gate) while building the
/// `ALTER TABLE ADD COLUMN` statement, which rejects it. That handler error
/// propagates all the way back through `EventBus::dispatch` ->
/// `SqliteEventStore::append` -> `KernelLog::emit` -> `create_schema`'s
/// caller as an opaque `CollectionError::Internal("persistence error:
/// projection 'collection' failed to apply 'collection.created'")`.
///
/// So the net behavior is *safe* (no SQL injection, no half-created table —
/// the transaction rolls back per `registry.rs`'s dispatch doc comment), but
/// it is a real contract break: `validate_schema_doc` is supposed to be the
/// place a bad schema gets a clean, actionable `Validation` error citing the
/// offending field. Instead a schema that LOOKS accepted (passes the public
/// validation function) blows up several layers down with an `Internal`
/// error that gives the caller no idea which field or rule was violated —
/// and every other field name schema.rs's looser rule accepts but
/// ship_substrate's stricter rule rejects (`"a.b"`, unicode homoglyphs,
/// leading digits) hits the same failure mode. Fix belongs in
/// `schema.rs::is_safe_identifier` (should mirror `ship_substrate::is_safe_ident`
/// exactly), not in the projection.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn field_name_accepted_by_schema_validation_fails_opaquely_downstream() {
    let tmp = tempfile::tempdir().unwrap();
    let kernel = make_kernel(&tmp);
    let ctx = make_ctx(&tmp, kernel);
    let svc = make_service(ctx);

    // schema.rs::is_safe_identifier allows the hyphen — validate_schema_doc
    // (called inside create_schema via parse_schema_doc) does not reject it.
    assert!(is_safe_identifier("foo-bar"));

    let created = svc
        .create_schema(
            "hyphenated",
            &serde_json::to_vec(&json!({
                "id": "hyphenated",
                "fields": { "foo-bar": { "type": "string" } }
            }))
            .unwrap(),
        )
        .await;

    let err = created.expect_err(
        "create_schema unexpectedly succeeded for a field name check_ident should reject \
         — the schema.rs/ship_substrate identifier-rule mismatch may have been fixed; \
         if so this test (and the finding it documents) is stale",
    );
    let msg = err.to_string();
    assert!(
        msg.contains("internal") && msg.contains("collection.created"),
        "expected the opaque downstream Internal error this finding documents, got: {msg}"
    );
    // The clean, actionable error `validate_schema_doc` SHOULD have produced
    // instead (a `Validation` naming the field) never happens — proving the
    // two identifier rules are out of sync at the point that matters.
    assert!(
        !matches!(err, ship_collection::error::CollectionError::Validation(_)),
        "this SHOULD be a Validation error naming the bad field, but the mismatch \
         between schema.rs and ship_substrate's identifier rules means it isn't"
    );
}

// ── translate_ast table/column identifiers ────────────────────────────────

#[test]
fn translate_ast_rejects_malicious_table_names() {
    for payload in MALICIOUS_IDENTS {
        let ast = json!({
            "kind": "SelectQueryNode",
            "from": {
                "kind": "FromNode",
                "froms": [{
                    "kind": "TableNode",
                    "table": {
                        "kind": "SchemableIdentifierNode",
                        "identifier": { "kind": "IdentifierNode", "name": payload }
                    }
                }]
            }
        });
        let result =
            ship_substrate::query::translate_ast(&serde_json::to_vec(&ast).unwrap(), None, 50);
        assert!(result.is_err(), "table name {payload:?} must be rejected");
    }
}

#[test]
fn translate_ast_rejects_malicious_column_names_in_where() {
    for payload in MALICIOUS_IDENTS {
        if payload.is_empty() {
            continue; // empty column name can't even be extracted; covered by the table case.
        }
        let ast = json!({
            "kind": "SelectQueryNode",
            "from": {
                "kind": "FromNode",
                "froms": [{
                    "kind": "TableNode",
                    "table": {
                        "kind": "SchemableIdentifierNode",
                        "identifier": { "kind": "IdentifierNode", "name": "tasks" }
                    }
                }]
            },
            "where": {
                "kind": "WhereNode",
                "where": {
                    "kind": "BinaryOperationNode",
                    "op": { "kind": "OperatorNode", "operator": "=" },
                    "leftOperand": {
                        "kind": "ReferenceNode",
                        "column": {
                            "kind": "ColumnNode",
                            "column": { "kind": "IdentifierNode", "name": payload }
                        }
                    },
                    "rightOperand": {
                        "kind": "ValueNode",
                        "value": { "kind": "PrimitiveValueNode", "value": "x", "type": "string" }
                    }
                }
            }
        });
        let result =
            ship_substrate::query::translate_ast(&serde_json::to_vec(&ast).unwrap(), None, 50);
        assert!(result.is_err(), "WHERE column {payload:?} must be rejected");
    }
}

#[test]
fn translate_ast_rejects_malicious_order_by_column() {
    for payload in ["x; DROP TABLE items; --", "a\"b", "1abc"] {
        let ast = json!({
            "kind": "SelectQueryNode",
            "from": {
                "kind": "FromNode",
                "froms": [{
                    "kind": "TableNode",
                    "table": {
                        "kind": "SchemableIdentifierNode",
                        "identifier": { "kind": "IdentifierNode", "name": "tasks" }
                    }
                }]
            },
            "orderBy": {
                "kind": "OrderByNode",
                "items": [{
                    "orderBy": {
                        "kind": "OrderByItemNode",
                        "column": {
                            "kind": "ColumnNode",
                            "column": { "kind": "IdentifierNode", "name": payload }
                        }
                    },
                    "direction": "asc"
                }]
            }
        });
        let result =
            ship_substrate::query::translate_ast(&serde_json::to_vec(&ast).unwrap(), None, 50);
        assert!(
            result.is_err(),
            "ORDER BY column {payload:?} must be rejected"
        );
    }
}

// ── collection_id / item_id injection through the service layer ──────────

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn create_schema_rejects_malicious_collection_ids() {
    let tmp = tempfile::tempdir().unwrap();
    let kernel = make_kernel(&tmp);
    let ctx = make_ctx(&tmp, kernel);
    let svc = make_service(ctx);

    for payload in MALICIOUS_IDENTS {
        if payload.is_empty() {
            continue; // empty id has its own dedicated rejection path (schema.rs).
        }
        let schema_json = serde_json::to_vec(&json!({
            "id": payload,
            "fields": { "title": { "type": "string" } }
        }))
        .unwrap();
        let result = svc.create_schema(payload, &schema_json).await;
        // Whether rejected at schema validation (is_safe_identifier) or later
        // by the projection's check_ident, it must never end up materialized
        // as a literal `collection_<payload>` table. We assert the create
        // call itself is rejected up front — the strongest guarantee.
        assert!(
            result.is_err(),
            "create_schema must reject malicious collection id {payload:?}"
        );
    }
}
