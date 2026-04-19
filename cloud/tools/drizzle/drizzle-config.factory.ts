import { defineConfig, type Config } from 'drizzle-kit';
import { findDurableObjectSqliteDb } from './find-do-local-sqlite.ts';

interface DurableObjectDrizzleOptions {
  /** Name of the Durable Object class (e.g., 'Platform') */
  durableObjectClass: string;
  /** Worker/handler name (e.g., 'platform-handler') */
  workerName: string;
  /** Directory for migrations (relative to package root). Defaults to ./migrations */
  migrationsDir?: string;
  /** Path to schema files (relative to package root). Defaults to ./src/data/schema/** /* */
  schemaPath?: string;
}

export function createDurableObjectDrizzleConfig(options: DurableObjectDrizzleOptions): Config {
  const {
    durableObjectClass,
    workerName,
    migrationsDir = './migrations',
    schemaPath = './src/data/schema/**/*',
  } = options;

  const isProd = process.env.NODE_ENV === 'production';

  const config: Config = {
    out: migrationsDir,
    schema: schemaPath,
    dialect: 'sqlite',
  };

  if (isProd) {
    return defineConfig({
      ...config,
      driver: 'durable-sqlite',
    });
  } else {
    return defineConfig({
      ...config,
      dbCredentials: {
        url: findDurableObjectSqliteDb(workerName, durableObjectClass),
      },
    });
  }
}
