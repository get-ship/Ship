// Schema vocabulary for the typed-from-schema components in @ship/collections.
//
// The collection's schema is the single source every typed-from-schema
// component reads — table columns, the item form's fields + validation,
// query autocomplete, and result columns all derive from it. The wire
// shape (`ship.collections.state({collection}).fields`) is the same
// record you see decoded in `.artifacts/design-drop/design/domains/
// collections/fixtures/collections.demo.json` (notes, tasks, contacts …).
//
// This file pins the TypeScript types those components import. The
// `text | integer | real | bool | timestamp | json` union is the
// canonical vocab for launch; relations and arrays are deferred to a
// later schema revision.

/** The six launch-stage field types — see spec.md §"The one hard
 *  requirement: typed-from-schema". */
export type FieldType = 'text' | 'integer' | 'real' | 'bool' | 'timestamp' | 'json'

/** A single schema field. Mirrors the decoded shape the daemon returns
 *  from `ship.collections.state`/`detail`. */
export interface FieldDef {
  /** Field name. Unique within a schema. Used as the row column key
   *  and the form field key. */
  name: string
  /** One of the six type vocabulary members. */
  type: FieldType
  /** True for fields that must be present and non-null on create +
   *  update. The item editor highlights them in the form. */
  required?: boolean
  /** True for fields backed by an indexable projection. */
  indexed?: boolean
  /** True for fields with a uniqueness constraint. */
  unique?: boolean
  /** Coerced into the matching type on create when the field is
   *  absent. Stored as `unknown` — the type-coercion is the schema
   *  editor's job. */
  default?: unknown
}

/** The schema for one collection. The `system` flag mirrors the
 *  daemon's read-only collections (feedback, annotations) and gates
 *  the schema editor + delete verb in the built-in view. */
export interface SchemaDef {
  /** Human title — the column header on the picker, the window
   *  title on the editor. */
  title: string
  fields: FieldDef[]
  /** Read-only collections (built-in feedback/annotation stores).
   *  When true the schema editor + delete verb stay hidden. */
  system?: boolean
}

/** A row's typed payload (one item, no envelope fields). The envelope
 *  (id, created_at, updated_at) is the substrate's; this is just the
 *  schema-validated record. */
export type RowData = Record<string, unknown>

/** Decide whether a value matches a field's type. Used by the item
 *  editor for client-side validation (mirrors the server rejection
 *  the spec calls out). Returns `true` when valid, an error code
 *  string when not. */
export function validateFieldValue(
  field: FieldDef,
  value: unknown,
): true | 'required' | 'wrong-type' {
  if (value === undefined || value === null) {
    return field.required ? 'required' : true
  }
  switch (field.type) {
    case 'text':
      return typeof value === 'string' ? true : 'wrong-type'
    case 'integer':
      return typeof value === 'number' && Number.isInteger(value) ? true : 'wrong-type'
    case 'real':
      return typeof value === 'number' ? true : 'wrong-type'
    case 'bool':
      return typeof value === 'boolean' ? true : 'wrong-type'
    case 'timestamp':
      return typeof value === 'string' ? true : 'wrong-type'
    case 'json':
      return true
  }
}
