import { migrate } from 'drizzle-orm/durable-sqlite/migrator';
import { drizzle } from 'drizzle-orm/durable-sqlite';
import migrations from '../../migrations/migrations.js';
import * as schema from './schema/index.js';

export async function initializeProjectDB(storage: DurableObjectStorage) {
  const db = drizzle<typeof schema.project_schema>(storage);
  await migrate(db, migrations);
}
