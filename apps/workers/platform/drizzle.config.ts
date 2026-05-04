import { createDurableObjectDrizzleConfig } from '../../../tools/drizzle/drizzle-config.factory';

export default createDurableObjectDrizzleConfig({
  durableObjectClass: 'Platform',
  workerName: 'platform-handler',
  migrationsDir: './migrations',
  schemaPath: './src/data/schema/**/*',
});
