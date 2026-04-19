import { env, runInDurableObject } from 'cloudflare:test';
import { drizzle } from 'drizzle-orm/durable-sqlite';
import { migrate } from 'drizzle-orm/durable-sqlite/migrator';
import { platform_schema } from '../src/data/schema/index.js';
import migrations from '../migrations/migrations.js';
import type { EventEnvelope } from '@ship-cloud/types';

export function makeEvent(overrides: Partial<EventEnvelope> = {}): EventEnvelope {
  return {
    id: crypto.randomUUID(),
    event_type: 'workspace.created',
    entity_id: crypto.randomUUID(),
    actor: 'ship',
    payload_json: JSON.stringify({ name: 'test-workspace' }),
    version: 1,
    elevated: false,
    created_at: new Date().toISOString(),
    ...overrides,
  };
}

export async function createMigratedStub() {
  const id = env.PLATFORM.idFromName(`test-${crypto.randomUUID()}`);
  const stub = env.PLATFORM.get(id);

  await runInDurableObject(stub, async (_instance: any, state: any) => {
    const db = drizzle(state.storage, { schema: platform_schema });
    await migrate(db, migrations);
  });

  return stub;
}

export { env, runInDurableObject, drizzle, platform_schema };
