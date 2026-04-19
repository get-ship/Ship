import { migrate } from 'drizzle-orm/durable-sqlite/migrator';
import { drizzle } from 'drizzle-orm/durable-sqlite';
import migrations from '../../migrations/migrations.js';
import * as schema from './schema/index.js';

export async function initializeWorkspaceDB(storage: DurableObjectStorage) {
  const db = drizzle<typeof schema.workspace_schema>(storage);
  await migrate(db, migrations);
}
