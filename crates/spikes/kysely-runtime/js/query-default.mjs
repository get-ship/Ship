// Default test query — exercises typed Kysely API on a notes table.
// This is what an agent would author.
import { Kysely, SqliteDialect } from 'kysely'

const dialect = new SqliteDialect({
  database: {
    prepare: () => { throw new Error('client never executes') },
    close: () => {},
  },
})

const db = new Kysely({ dialect })

export const query = db
  .selectFrom('notes')
  .selectAll()
  .where('priority', '>=', 3)
  .where('pinned', '=', 1)
  .orderBy('priority', 'desc')
  .limit(10)
