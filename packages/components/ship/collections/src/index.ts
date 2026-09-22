// @ship/collections — Ship Collections pack (ship-* tags for the
// collections domain: list, table, item detail/editor, schema editor,
// query editor/runner, picker).
//
// Each <tag>.sail calls customElements.define() on import, so a
// single `import '@ship/collections'` registers every tag globally.
//
// Host requirement: the host (shell main.ts) must call setShipClient()
// before any of these tags mount, otherwise the lazy `ship` Proxy will
// throw on first access. The pack composes @ship/sail-primitives
// (<sui-table>, <sui-input>, <sui-textarea>, <sui-checkbox>, <sui-button>,
// <sui-error-state>, <sui-empty-state>, <sui-skeleton>, <sui-json-view>),
// a declared dependency the host registers via `import '@ship/sail-primitives'`.
//
// Composition order:
//   1. Schema vocabulary types (consumed by every typed-from-schema
//      component; exporting from the index keeps the surface single).
//   2. Leaves — list · picker · item detail · query editor/runner.
//   3. Composites — table · schema editor · item editor (each pulls
//      in the typed-from-schema derivation its siblings expose).

// Schema vocabulary — the field-type union + FieldDef/SchemaDef that
// the typed-from-schema components import. Re-exporting here means
// consumers get the full surface from one entry point.
export type { FieldDef, FieldType, RowData, SchemaDef } from './schema'
export { validateFieldValue } from './schema'

// Leaves.
import './ship-collection-list/ship-collection-list.compiled.js'
import './ship-collection-picker/ship-collection-picker.compiled.js'
import './ship-item-detail/ship-item-detail.compiled.js'
import './ship-query-editor/ship-query-editor.compiled.js'
import './ship-query-runner/ship-query-runner.compiled.js'

// Composites.
import './ship-collection-table/ship-collection-table.compiled.js'
import './ship-item-editor/ship-item-editor.compiled.js'
import './ship-schema-editor/ship-schema-editor.compiled.js'
