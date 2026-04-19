import { env, runInDurableObject } from 'cloudflare:test';
import { drizzle } from 'drizzle-orm/durable-sqlite';
import { migrate } from 'drizzle-orm/durable-sqlite/migrator';
import { workspace_schema } from '../src/data/schema/index.js';
import migrations from '../migrations/migrations.js';
import type { EventEnvelope } from '@ship-cloud/types';

export function makeEvent(overrides: Partial<EventEnvelope> = {}): EventEnvelope {
  return {
    id: crypto.randomUUID(),
    event_type: 'actor.created',
    entity_id: crypto.randomUUID(),
    actor: 'ship',
    payload_json: JSON.stringify({ name: 'test-actor' }),
    version: 1,
    causation_id: null,
    workspace_id: null,
    session_id: null,
    actor_id: null,
    parent_actor_id: null,
    target_actor_id: null,
    elevated: false,
    created_at: new Date().toISOString(),
    ...overrides,
  };
}

export async function createMigratedStub() {
  const id = env.WORKSPACE.idFromName(`test-${crypto.randomUUID()}`);
  const stub = env.WORKSPACE.get(id);

  await runInDurableObject(stub, async (_instance: any, state: any) => {
    const db = drizzle(state.storage, { schema: workspace_schema });
    await migrate(db, migrations);
  });

  return stub;
}

export { env, runInDurableObject, drizzle, workspace_schema };
