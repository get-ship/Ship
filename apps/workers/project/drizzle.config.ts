import { createDurableObjectDrizzleConfig } from '../../../tools/drizzle/drizzle-config.factory';

export default createDurableObjectDrizzleConfig({
  durableObjectClass: 'Workspace',
  workerName: 'workspace-handler',
  migrationsDir: './migrations',
  schemaPath: './src/data/schema/**/*',
});
