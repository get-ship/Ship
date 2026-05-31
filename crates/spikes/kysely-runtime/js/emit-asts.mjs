// Spike: emit several Kysely OperationNode ASTs as JSON for the Rust executor
// to parse + run. Stdout = JSON document with named queries.
//
// Run: bun emit-asts.mjs > asts.json
import { Kysely, SqliteDialect, sql } from 'kysely'

// Hand-rolled tiny "driver" that never actually opens a DB — we only need the
// AST. Kysely requires a dialect; we give it sqlite's because the SQL emission
// shape matches what our server will run.
const dialect = new SqliteDialect({
  database: {
    // Stub: Kysely's Sqlite dialect needs a Database object but never calls
    // it unless we execute(). We compile, we don't execute.
    prepare: () => { throw new Error('spike never executes client-side') },
    close: () => {},
  },
})

// Mock Database interface for typed query building.
/** @typedef {{ id: string, title: string, priority: number, pinned: number, body: string }} NoteRow */
/** @typedef {{ notes: NoteRow }} Database */

const db = /** @type {Kysely<Database>} */ (new Kysely({ dialect }))

function captureAst(label, queryBuilder) {
  // toOperationNode() returns the unflattened AST. compile() returns the SQL.
  // We send BOTH so the Rust side can cross-check during dev (production sends AST only).
  const node = queryBuilder.toOperationNode()
  const compiled = queryBuilder.compile()
  return {
    label,
    operationNode: node,
    expectedSql: compiled.sql,
    expectedParameters: compiled.parameters,
  }
}

const asts = [
  // 1. SELECT * FROM notes
  captureAst('select_all', db.selectFrom('notes').selectAll()),

  // 2. SELECT id, title FROM notes WHERE priority >= 3
  captureAst(
    'select_with_where_gte',
    db.selectFrom('notes').select(['id', 'title']).where('priority', '>=', 3),
  ),

  // 3. SELECT * FROM notes WHERE pinned = 1 AND priority >= 3
  captureAst(
    'select_with_and_filter',
    db.selectFrom('notes')
      .selectAll()
      .where('pinned', '=', 1)
      .where('priority', '>=', 3),
  ),

  // 4. SELECT * FROM notes ORDER BY priority DESC LIMIT 5
  captureAst(
    'select_with_order_limit',
    db.selectFrom('notes').selectAll().orderBy('priority', 'desc').limit(5),
  ),

  // 5. SELECT * FROM notes WHERE title LIKE '%spike%'
  captureAst(
    'select_with_like',
    db.selectFrom('notes').selectAll().where('title', 'like', '%spike%'),
  ),
]

process.stdout.write(JSON.stringify(asts, null, 2))
