//! Scenario 4 — query contract fuzz.
//!
//! `ship_substrate::query::translate_ast` translates a whitelisted subset of
//! the Kysely AST. Everything else must come back `Err(Unsupported)`. This
//! file exercises the translator directly as a library (no DB), per the
//! dispatch's "call the public ship_substrate query API as a lib" scoping.

use serde_json::json;
use ship_substrate::query::translate_ast;

fn simple_from(table: &str) -> serde_json::Value {
    json!({
        "kind": "FromNode",
        "froms": [{
            "kind": "TableNode",
            "table": {
                "kind": "SchemableIdentifierNode",
                "identifier": { "kind": "IdentifierNode", "name": table }
            }
        }]
    })
}

fn assert_rejected(ast: &serde_json::Value, why: &str) {
    let bytes = serde_json::to_vec(ast).unwrap();
    let result = translate_ast(&bytes, None, 50);
    assert!(result.is_err(), "{why}: expected Unsupported, got {result:?}");
}

// ── happy path sanity (whitelisted shapes translate) ──────────────────────

#[test]
fn valid_select_with_eq_where_translates() {
    let ast = json!({
        "kind": "SelectQueryNode",
        "from": simple_from("tasks"),
        "where": {
            "kind": "WhereNode",
            "where": {
                "kind": "BinaryOperationNode",
                "op": { "kind": "OperatorNode", "operator": "=" },
                "leftOperand": {
                    "kind": "ReferenceNode",
                    "column": { "kind": "ColumnNode", "column": { "kind": "IdentifierNode", "name": "title" } }
                },
                "rightOperand": {
                    "kind": "ValueNode",
                    "value": { "kind": "PrimitiveValueNode", "value": "Alpha", "type": "string" }
                }
            }
        }
    });
    let result = translate_ast(&serde_json::to_vec(&ast).unwrap(), None, 50);
    assert!(result.is_ok(), "{result:?}");
    let q = result.unwrap();
    assert_eq!(q.table, "tasks");
    assert_eq!(q.filters.len(), 1);
}

#[test]
fn valid_and_chain_of_multiple_operators_translates() {
    let ast = json!({
        "kind": "SelectQueryNode",
        "from": simple_from("tasks"),
        "where": {
            "kind": "WhereNode",
            "where": {
                "kind": "AndNode",
                "left": {
                    "kind": "BinaryOperationNode",
                    "op": { "kind": "OperatorNode", "operator": ">" },
                    "leftOperand": { "kind": "ReferenceNode", "column": { "kind": "ColumnNode", "column": { "kind": "IdentifierNode", "name": "rank" } } },
                    "rightOperand": { "kind": "ValueNode", "value": { "kind": "PrimitiveValueNode", "value": 1, "type": "number" } }
                },
                "right": {
                    "kind": "BinaryOperationNode",
                    "op": { "kind": "OperatorNode", "operator": "<=" },
                    "leftOperand": { "kind": "ReferenceNode", "column": { "kind": "ColumnNode", "column": { "kind": "IdentifierNode", "name": "rank" } } },
                    "rightOperand": { "kind": "ValueNode", "value": { "kind": "PrimitiveValueNode", "value": 100, "type": "number" } }
                }
            }
        }
    });
    let q = translate_ast(&serde_json::to_vec(&ast).unwrap(), None, 50).unwrap();
    assert_eq!(q.filters.len(), 2);
}

#[test]
fn valid_in_predicate_translates() {
    let ast = json!({
        "kind": "SelectQueryNode",
        "from": simple_from("tasks"),
        "where": {
            "kind": "WhereNode",
            "where": {
                "kind": "BinaryOperationNode",
                "op": { "kind": "OperatorNode", "operator": "in" },
                "leftOperand": { "kind": "ReferenceNode", "column": { "kind": "ColumnNode", "column": { "kind": "IdentifierNode", "name": "status" } } },
                "rightOperand": {
                    "kind": "PrimitiveValueListNode",
                    "values": ["open", "in_progress"]
                }
            }
        }
    });
    let q = translate_ast(&serde_json::to_vec(&ast).unwrap(), None, 50).unwrap();
    assert_eq!(q.filters.len(), 1);
}

// ── forbidden top-level statement kinds ───────────────────────────────────

#[test]
fn delete_query_node_rejected() {
    assert_rejected(&json!({ "kind": "DeleteQueryNode", "from": simple_from("tasks") }), "DeleteQueryNode");
}

#[test]
fn update_query_node_rejected() {
    assert_rejected(&json!({ "kind": "UpdateQueryNode", "table": simple_from("tasks") }), "UpdateQueryNode");
}

#[test]
fn insert_query_node_rejected() {
    assert_rejected(&json!({ "kind": "InsertQueryNode", "into": simple_from("tasks") }), "InsertQueryNode");
}

#[test]
fn raw_top_level_node_rejected() {
    assert_rejected(&json!({ "kind": "RawNode", "sql": "SELECT 1" }), "top-level RawNode");
}

// ── forbidden nodes buried anywhere in the tree ───────────────────────────

#[test]
fn raw_node_buried_in_where_rejected() {
    assert_rejected(
        &json!({
            "kind": "SelectQueryNode",
            "from": simple_from("tasks"),
            "where": { "kind": "WhereNode", "where": { "kind": "RawNode", "sql": "1=1; DROP TABLE tasks" } }
        }),
        "RawNode in WHERE",
    );
}

#[test]
fn raw_node_buried_in_value_operand_rejected() {
    assert_rejected(
        &json!({
            "kind": "SelectQueryNode",
            "from": simple_from("tasks"),
            "where": {
                "kind": "WhereNode",
                "where": {
                    "kind": "BinaryOperationNode",
                    "op": { "kind": "OperatorNode", "operator": "=" },
                    "leftOperand": { "kind": "ReferenceNode", "column": { "kind": "ColumnNode", "column": { "kind": "IdentifierNode", "name": "title" } } },
                    "rightOperand": { "kind": "RawNode", "sql": "(SELECT password FROM secrets)" }
                }
            }
        }),
        "RawNode as a value operand",
    );
}

#[test]
fn join_node_rejected() {
    assert_rejected(
        &json!({
            "kind": "SelectQueryNode",
            "from": simple_from("tasks"),
            "joins": [{ "kind": "JoinNode", "table": simple_from("secrets") }]
        }),
        "JoinNode",
    );
}

#[test]
fn union_intersect_except_rejected() {
    for kind in ["UnionNode", "IntersectNode", "ExceptNode"] {
        assert_rejected(
            &json!({
                "kind": "SelectQueryNode",
                "from": simple_from("tasks"),
                "setOperations": [{ "kind": kind, "expression": { "kind": "SelectQueryNode", "from": simple_from("secrets") } }]
            }),
            kind,
        );
    }
}

#[test]
fn group_by_and_having_rejected() {
    assert_rejected(
        &json!({
            "kind": "SelectQueryNode",
            "from": simple_from("tasks"),
            "groupBy": { "kind": "GroupByNode", "items": [] }
        }),
        "GroupByNode",
    );
    assert_rejected(
        &json!({
            "kind": "SelectQueryNode",
            "from": simple_from("tasks"),
            "having": { "kind": "HavingNode" }
        }),
        "HavingNode",
    );
}

#[test]
fn subquery_node_rejected() {
    assert_rejected(
        &json!({
            "kind": "SelectQueryNode",
            "from": {
                "kind": "FromNode",
                "froms": [{ "kind": "SubqueryNode", "query": { "kind": "SelectQueryNode", "from": simple_from("secrets") } }]
            }
        }),
        "SubqueryNode in FROM",
    );
}

#[test]
fn function_call_node_in_where_rejected() {
    // Function calls aren't a documented BinaryOperationNode leftOperand
    // shape at all — the extractor should fail closed (Unsupported), not
    // silently coerce it into something.
    assert_rejected(
        &json!({
            "kind": "SelectQueryNode",
            "from": simple_from("tasks"),
            "where": {
                "kind": "WhereNode",
                "where": {
                    "kind": "BinaryOperationNode",
                    "op": { "kind": "OperatorNode", "operator": "=" },
                    "leftOperand": { "kind": "FunctionNode", "function": "lower", "arguments": [] },
                    "rightOperand": { "kind": "ValueNode", "value": { "kind": "PrimitiveValueNode", "value": "x", "type": "string" } }
                }
            }
        }),
        "FunctionNode as leftOperand",
    );
}

#[test]
fn or_chain_rejected() {
    assert_rejected(
        &json!({
            "kind": "SelectQueryNode",
            "from": simple_from("tasks"),
            "where": {
                "kind": "WhereNode",
                "where": {
                    "kind": "OrNode",
                    "left": {
                        "kind": "BinaryOperationNode",
                        "op": { "kind": "OperatorNode", "operator": "=" },
                        "leftOperand": { "kind": "ReferenceNode", "column": { "kind": "ColumnNode", "column": { "kind": "IdentifierNode", "name": "title" } } },
                        "rightOperand": { "kind": "ValueNode", "value": { "kind": "PrimitiveValueNode", "value": "a", "type": "string" } }
                    },
                    "right": {
                        "kind": "BinaryOperationNode",
                        "op": { "kind": "OperatorNode", "operator": "=" },
                        "leftOperand": { "kind": "ReferenceNode", "column": { "kind": "ColumnNode", "column": { "kind": "IdentifierNode", "name": "title" } } },
                        "rightOperand": { "kind": "ValueNode", "value": { "kind": "PrimitiveValueNode", "value": "b", "type": "string" } }
                    }
                }
            }
        }),
        "OrNode",
    );
}

#[test]
fn not_node_rejected() {
    assert_rejected(
        &json!({
            "kind": "SelectQueryNode",
            "from": simple_from("tasks"),
            "where": {
                "kind": "WhereNode",
                "where": {
                    "kind": "NotNode",
                    "expr": {
                        "kind": "BinaryOperationNode",
                        "op": { "kind": "OperatorNode", "operator": "=" },
                        "leftOperand": { "kind": "ReferenceNode", "column": { "kind": "ColumnNode", "column": { "kind": "IdentifierNode", "name": "title" } } },
                        "rightOperand": { "kind": "ValueNode", "value": { "kind": "PrimitiveValueNode", "value": "a", "type": "string" } }
                    }
                }
            }
        }),
        "NotNode",
    );
}

#[test]
fn multi_table_from_rejected() {
    assert_rejected(
        &json!({
            "kind": "SelectQueryNode",
            "from": {
                "kind": "FromNode",
                "froms": [
                    { "kind": "TableNode", "table": { "kind": "SchemableIdentifierNode", "identifier": { "kind": "IdentifierNode", "name": "tasks" } } },
                    { "kind": "TableNode", "table": { "kind": "SchemableIdentifierNode", "identifier": { "kind": "IdentifierNode", "name": "secrets" } } }
                ]
            }
        }),
        "multi-table FROM",
    );
}

#[test]
fn operator_outside_whitelist_rejected() {
    for op in ["!=", "<>", "like", "LIKE", "is", "between", "&&", "||"] {
        assert_rejected(
            &json!({
                "kind": "SelectQueryNode",
                "from": simple_from("tasks"),
                "where": {
                    "kind": "WhereNode",
                    "where": {
                        "kind": "BinaryOperationNode",
                        "op": { "kind": "OperatorNode", "operator": op },
                        "leftOperand": { "kind": "ReferenceNode", "column": { "kind": "ColumnNode", "column": { "kind": "IdentifierNode", "name": "title" } } },
                        "rightOperand": { "kind": "ValueNode", "value": { "kind": "PrimitiveValueNode", "value": "x", "type": "string" } }
                    }
                }
            }),
            &format!("operator {op:?}"),
        );
    }
}

// ── malformed / adversarial JSON shapes ───────────────────────────────────

#[test]
fn empty_object_rejected() {
    assert_rejected(&json!({}), "empty object (no kind)");
}

#[test]
fn missing_from_clause_rejected() {
    assert_rejected(&json!({ "kind": "SelectQueryNode" }), "missing FROM");
}

#[test]
fn non_json_bytes_rejected() {
    let result = translate_ast(b"not json at all {{{", None, 50);
    assert!(result.is_err());
}

/// FINDING: a right-leaning `AndNode` chain ~2000 deep stack-overflows and
/// SIGABRTs the process. Two unbounded recursions compound here: `translate_ast`
/// first walks the WHOLE AST via `reject_forbidden_nodes` (recursing into
/// every object/array value with no depth cap), then, once past that,
/// `collect_predicates` recurses once per `AndNode` link (also uncapped).
/// `query_items_ast` takes `ast_bytes: &[u8]` straight from the caller with
/// no size or depth limit before ever reaching `translate_ast`, so a single
/// malicious inline-AST query payload — not even a large one; ~2000 nested
/// JSON objects is well under most body-size caps — can crash the daemon
/// process. This is a direct break of the "fuzz translate_ast ... must not
/// panic" spirit of master property 3 / scenario 4: the whitelist correctly
/// rejects disallowed node kinds, but never bounds recursion depth, so a
/// well-formed-looking (all whitelisted node kinds!) WHERE clause is itself
/// an attack. Left `#[ignore]` because a stack overflow SIGABRTs the test
/// binary (no `Err` to assert — the process dies before `translate_ast`
/// returns).
#[ignore = "FINDING: ~2000-deep AndNode chain (or equally deep nesting anywhere reject_forbidden_nodes walks) stack-overflows translate_ast (SIGABRT) — no recursion depth limit on caller-controlled inline-AST query bytes"]
#[test]
fn deeply_nested_and_chain_does_not_panic() {
    // Build a right-leaning AndNode chain 2000 deep — must either translate
    // (flattening into 2000 predicates) or reject cleanly; must not panic or
    // stack-overflow on a plausible-sized adversarial WHERE clause.
    fn leaf(n: i64) -> serde_json::Value {
        json!({
            "kind": "BinaryOperationNode",
            "op": { "kind": "OperatorNode", "operator": "=" },
            "leftOperand": { "kind": "ReferenceNode", "column": { "kind": "ColumnNode", "column": { "kind": "IdentifierNode", "name": "title" } } },
            "rightOperand": { "kind": "ValueNode", "value": { "kind": "PrimitiveValueNode", "value": n, "type": "number" } }
        })
    }
    let mut expr = leaf(0);
    for i in 1..2000 {
        expr = json!({ "kind": "AndNode", "left": leaf(i), "right": expr });
    }
    let ast = json!({
        "kind": "SelectQueryNode",
        "from": simple_from("tasks"),
        "where": { "kind": "WhereNode", "where": expr }
    });
    // Not asserting Ok/Err — only that it returns rather than crashing.
    let _ = translate_ast(&serde_json::to_vec(&ast).unwrap(), None, 50);
}

// ── value-injection attempts through WHERE operands ───────────────────────

#[test]
fn sql_text_smuggled_as_a_bound_value_is_carried_as_a_plain_string_not_sql() {
    // The classic injection payload placed where a legitimate string value
    // goes. translate_ast must accept it as data (a Predicate::value), never
    // interpret it — the executor (single_backend.rs) binds it as a
    // parameter, never string-interpolates it. This proves the AST layer at
    // least treats it as an opaque value rather than doing anything special.
    let payload = "x'; DROP TABLE tasks; --";
    let ast = json!({
        "kind": "SelectQueryNode",
        "from": simple_from("tasks"),
        "where": {
            "kind": "WhereNode",
            "where": {
                "kind": "BinaryOperationNode",
                "op": { "kind": "OperatorNode", "operator": "=" },
                "leftOperand": { "kind": "ReferenceNode", "column": { "kind": "ColumnNode", "column": { "kind": "IdentifierNode", "name": "title" } } },
                "rightOperand": { "kind": "ValueNode", "value": { "kind": "PrimitiveValueNode", "value": payload, "type": "string" } }
            }
        }
    });
    let q = translate_ast(&serde_json::to_vec(&ast).unwrap(), None, 50).unwrap();
    assert_eq!(q.filters.len(), 1);
    assert_eq!(
        q.filters[0].value,
        serde_json::Value::String(payload.to_string()),
        "the payload must survive as an exact, unmodified string value"
    );
}

#[test]
fn in_list_with_injection_payloads_carries_them_as_opaque_values() {
    let ast = json!({
        "kind": "SelectQueryNode",
        "from": simple_from("tasks"),
        "where": {
            "kind": "WhereNode",
            "where": {
                "kind": "BinaryOperationNode",
                "op": { "kind": "OperatorNode", "operator": "in" },
                "leftOperand": { "kind": "ReferenceNode", "column": { "kind": "ColumnNode", "column": { "kind": "IdentifierNode", "name": "status" } } },
                "rightOperand": {
                    "kind": "PrimitiveValueListNode",
                    "values": ["a'); DROP TABLE tasks; --", "1 OR 1=1", "\0\0\0"]
                }
            }
        }
    });
    let q = translate_ast(&serde_json::to_vec(&ast).unwrap(), None, 50).unwrap();
    assert_eq!(q.filters.len(), 1);
    let arr = q.filters[0].value.as_array().unwrap();
    assert_eq!(arr.len(), 3);
}

#[test]
fn limit_is_clamped_not_trusted_verbatim() {
    let ast = json!({ "kind": "SelectQueryNode", "from": simple_from("tasks") });
    let q = translate_ast(&serde_json::to_vec(&ast).unwrap(), None, u32::MAX).unwrap();
    assert!(q.limit <= 500, "limit must be clamped to the server-side max, got {}", q.limit);
}

#[test]
fn zero_limit_falls_back_to_default_not_unbounded() {
    let ast = json!({ "kind": "SelectQueryNode", "from": simple_from("tasks") });
    let q = translate_ast(&serde_json::to_vec(&ast).unwrap(), None, 0).unwrap();
    assert_eq!(q.limit, 50);
}
