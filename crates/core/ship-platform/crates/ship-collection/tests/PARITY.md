# ship-collection — parity report

## Summary

| | Count |
|---|---|
| PORT (tests added in parity.rs) | 45 |
| IGNORED/GAP (in parity.rs, `#[ignore]`) | 6 |
| OBSOLETE (old plumbing gone, omitted) | 22 |

Old baseline: **~160 old tests** across schema_tests.rs, tests_integration.rs,
integration_builtin.rs, integration_schema.rs, integration_service.rs,
integration_validation.rs.

New baseline (before this job): **12 tests** in behavior.rs.

New baseline (after this job): **12 (behavior.rs) + 45 (parity.rs) = 57 tests**.

---

## PARITY-GAPs (operator dispositions required)

| # | Gap | Impact |
|---|-----|--------|
| 1 | **Array element type validation** — old impl validated each element in an array field with index-path errors (`tags[1] must be a string`). New impl only checks that the field is an array. | Low. Array fields are permissive by design in the new schema model. |
| 2 | **Object required\_keys validation** — old impl validated `required_keys` on object fields (`author.name is required`). New impl only checks that the field is an object. | Low. Object properties are permissive by design. |
| 3 | **gte/lt/lte filter operator** — old impl supported `priority__gte: 2` style filters. New AST translator only supports `=`. | Medium. Range queries on numeric fields require AST extension. |
| 4 | **IN filter operator** — old impl supported `status__in: ["todo", "doing"]`. New impl has no IN-query support. | Medium. Multi-value equality queries require AST extension. |
| 5 | **Sort descending** — old impl supported `sort.insert("priority", -1)`. New `Query::sort_asc` is ascending-only; AST ORDER BY loses direction. | Low. Descending sort requires a direction flag on Query. |
| 6 | **Update merge/patch semantics** — old `update_item` merged partial data (passing `{"status": "done"}` preserved existing `title`). New impl replaces the full item. | Medium. Callers who relied on partial update must fetch+merge+resubmit. |

---

## OBSOLETE tests (not ported)

- `parse_rejects_items_on_non_array`, `parse_rejects_properties_on_non_object`,
  `parse_rejects_required_key_not_in_properties` — `items`/`properties`/`required_keys`
  sub-schema fields are not part of the new `FieldDef` (simpler design).

- `array_of_strings_rejects_non_string_with_index_path`,
  `object_required_key_present_passes`, `object_missing_required_key_rejected_with_key_name`,
  `object_property_wrong_type_rejected`, `deeply_nested_array_of_arrays_path_cited`,
  `array_of_objects_required_key_path_cited`, `array_of_objects_enum_value_validated`,
  `collects_multiple_nested_errors`, `round_trip_serde_preserves_nested_schema` —
  nested sub-schema validation removed from new design (→ GAPs #1 and #2).

- `load_schemas_from_empty_dir`, `load_schemas_from_dir`, `load_schemas_skips_non_schema_dir`,
  `load_schemas_merged_combines_global_and_project`, `load_schemas_merged_project_shadows_global`,
  `load_schemas_merged_no_project_equals_global`, `collections_schema_load_*`,
  `collections_schema_merged_*` — new crate uses `SchemaRegistry` (in-memory seeded from
  builtins + `_schema_registry` DataStore table), not filesystem JSONC loading.

- `collections_builtin_register_creates_all_tables`,
  `collections_builtin_register_is_idempotent` — no public `register()` API; table
  creation is triggered by `create_schema`.

- `collections_builtin_adrs_schema_parses` — `adrs` schema does not exist in new builtins.

- `query_with_sort_descending`, `query_with_limit_and_offset` — descending sort not
  supported (→ GAP #5); offset replaced by cursor pagination in new design.
