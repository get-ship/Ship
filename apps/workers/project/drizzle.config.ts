import { createDurableObjectDrizzleConfig } from '../drizzle/drizzle-config.factory';

export default createDurableObjectDrizzleConfig({
  durableObjectClass: 'Project',
  workerName: 'project-handler',
  migrationsDir: './migrations',
  schemaPath: './src/data/schema/**/*',
});
